#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::{debug, info};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::{Duration, Instant, Ticker, Timer};
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::{Dimensions, Point, RgbColor, Size},
    primitives::Rectangle,
};
use embedded_text::{
    TextBox,
    alignment::{HorizontalAlignment, VerticalAlignment},
    style::TextBoxStyleBuilder,
};
use esp_rtos::embassy::Executor;
use panic_rtt_target as _;
use static_cell::StaticCell;
use tildagon::{
    bq25895::{self, Bq25895},
    button_collection::{ButtonEvent, ButtonState},
    esp_hal::{
        self,
        clock::CpuClock,
        interrupt::software::SoftwareInterruptControl,
        peripherals::{DMA_CH0, SPI2},
        rmt::Rmt,
        system::Stack,
        time::Rate,
        timer::timg::TimerGroup,
    },
    front::{
        FrontBoardLeds,
        leds::{BaseBoardLed, FrontLeds, HexpansionPortLed},
    },
    hexpansions::{HexpansionSlot, HexpansionSlotControl, HexpansionSlotEvent, HexpansionState},
    i2c::{SharedI2cBus, SharedI2cDevice, SystemI2cBus},
    led_power::OnboardLedPower,
    pins::PinControl,
    resources::*,
    smart_leds::{
        RGB8, SmartLedsWrite,
        hsv::{Hsv, hsv2rgb},
    },
    usb::{UsbPort, UsbSwitch},
};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let config = tildagon::esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = tildagon::esp_hal::init(config);
    let r = tildagon::split_resources!(p);

    esp_alloc::heap_allocator!(size: 64 * 1024);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024);

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timg0.timer0);

    static I2C_BUS: StaticCell<SharedI2cBus<tildagon::i2c::I2c>> = StaticCell::new();
    let (bus, _reset) = tildagon::i2c::i2c_bus(r.i2c).await;
    let i2c_bus = I2C_BUS.init(bus);

    static I2C_SYSTEM: StaticCell<SharedI2cBus<tildagon::i2c::SystemI2cBus>> = StaticCell::new();
    let i2c_system = I2C_SYSTEM.init(tildagon::i2c::system_i2c_bus(i2c_bus));

    let mut pin_control = PinControl::new(i2c_system).await.unwrap();
    let pins = pin_control.pins();

    let mut usb_sw = UsbSwitch::new(pins.usb);
    usb_sw.set(UsbPort::In).await.unwrap();

    let mut buttons = tildagon::front::emf2024::SystemButtonCollection::new(pins.buttons);

    let mut hex_slots = HexpansionSlotControl::new(pins.hexpansion_detect)
        .await
        .unwrap();

    let rmt: Rmt<'_, esp_hal::Blocking> = Rmt::new(p.RMT, Rate::from_mhz(80)).unwrap();

    let mut led_power = OnboardLedPower::new(pins.led);
    led_power.set(true).await.unwrap();

    static APP_CORE_STACK: StaticCell<Stack<8192>> = StaticCell::new();
    let app_core_stack = APP_CORE_STACK.init(Stack::new());

    let sw_int = SoftwareInterruptControl::new(p.SW_INTERRUPT);
    esp_rtos::start_second_core(
        p.CPU_CTRL,
        sw_int.software_interrupt0,
        sw_int.software_interrupt1,
        app_core_stack,
        move || {
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                spawner.must_spawn(display_task(r.top_board, p.SPI2, p.DMA_CH0));
            });
        },
    );

    let bq = tildagon::power::new_bq25895(i2c_system);
    spawner.must_spawn(power_task(bq));

    spawner.must_spawn(led_task(r.led, rmt.channel0));
    spawner.must_spawn(button_logic_task());

    // A little time for other tasks to start.
    // Hacky as all fuck but good enough for a demo.
    // Use channels to indicate readiness properly, mkay.
    Timer::after_millis(500).await;

    let mut tick = Ticker::every(Duration::from_millis(100));
    let mut hex_control_sub = HEX_CONTROL_CHANNEL.subscriber().unwrap();
    let event_pub = EVENT_CHANNEL.publisher().unwrap();

    loop {
        match select(tick.next(), hex_control_sub.next_message()).await {
            Either::First(_) => {
                let regs = pin_control.read_system_bus_input_registers().await.unwrap();

                for event in buttons.update(&regs).unwrap() {
                    info!("Button event: {}", event);
                    event_pub.publish(Event::Button(event)).await;
                }

                for event in hex_slots.update(&regs) {
                    info!("Hexpansion event: {}", event);
                    event_pub.publish(Event::HexpansionSlot(event)).await;
                }
            }
            Either::Second(WaitResult::Lagged(_)) => panic!(),
            Either::Second(WaitResult::Message(msg)) => {
                hex_slots.set_enabled(msg.slot, msg.enable).await.unwrap();
            }
        }
    }
}

#[embassy_executor::task]
async fn power_task(mut bq: Bq25895<bq25895::Interface<SharedI2cDevice<SystemI2cBus>>>) {
    let mut tick = Ticker::every(Duration::from_millis(1000));

    loop {
        tick.next().await;

        info!("power stats @ {}s", Instant::now().as_secs());

        // Feed watchdog
        bq.reg_03()
            .modify_async(|r| r.set_wd_rst(bq25895::I2cWatchdogReset::Normal))
            .await
            .unwrap();

        // Set input current limit
        bq.reg_00()
            .modify_async(|r| r.set_iinlim(bq25895::InputCurrentLimit::try_new(600).unwrap()))
            .await
            .unwrap();

        // Request measurement
        bq.reg_02()
            .modify_async(|r| r.set_conv_start(bq25895::AdcConversionControl::Started))
            .await
            .unwrap();

        // Wait for measurement to finish
        'read: loop {
            let reg = bq.reg_02().read_async().await.unwrap();
            if reg.conv_start().unwrap() == bq25895::AdcConversionControl::Inactive {
                break 'read;
            }
            Timer::after_millis(100).await;
            debug!("Waiting for reading...");
        }

        let reg = bq.reg_02().read_async().await.unwrap();
        info!("conv start: {}", reg.conv_start().unwrap());
        info!("conv rate: {}", reg.conv_rate().unwrap());
        let reg = bq.reg_00().read_async().await.unwrap();
        info!("input current limit: {}", reg.iinlim().unwrap());
        let reg = bq.reg_0_b().read_async().await.unwrap();
        info!("Vbus stat: {}", reg.vbus_stat().unwrap());
        info!("Charging stat.: {}", reg.chrg_stat().unwrap());
        info!("PG: {}", reg.pg_stat().unwrap());
        let reg = bq.reg_0_c().read_async().await.unwrap();
        info!("WDT: {}", reg.watchdog_fault().unwrap());
        let reg = bq.reg_04().read_async().await.unwrap();
        info!("Ichg_lim: {}", reg.ichg().unwrap());
        let reg = bq.reg_11().read_async().await.unwrap();
        info!("Vbus: {}", reg.vbusv().unwrap());
        let reg = bq.reg_0_f().read_async().await.unwrap();
        info!("Vsys: {}", reg.sysv().unwrap());
        let reg = bq.reg_0_e().read_async().await.unwrap();
        info!("Vbat: {}", reg.batv().unwrap());
        let reg = bq.reg_12().read_async().await.unwrap();
        info!("Ichgr: {}", reg.ichgr().unwrap());
        let reg = bq.reg_14().read_async().await.unwrap();
        info!("ico: {}", reg.ico_optimized().unwrap());
    }
}

#[derive(Clone)]
enum Event {
    Button(ButtonEvent<tildagon::front::emf2024::SystemButton>),
    HexpansionSlot(HexpansionSlotEvent),
}

static EVENT_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Event, 12, 4, 4> =
    PubSubChannel::new();

#[derive(Clone)]
struct HexpansionControlMsg {
    slot: HexpansionSlot,
    enable: bool,
}

static HEX_CONTROL_CHANNEL: PubSubChannel<CriticalSectionRawMutex, HexpansionControlMsg, 6, 1, 1> =
    PubSubChannel::new();

use tildagon::front::FrontBoardDisplay;

#[embassy_executor::task]
async fn display_task(
    top_board: TopBoardResources<'static>,
    spi: SPI2<'static>,
    dma: DMA_CH0<'static>,
) {
    let mut display_buffer = [0_u8; 512];
    let mut display = <tildagon::front::Emf2024FrontBoard as FrontBoardDisplay>::Display::init(
        top_board,
        spi,
        dma,
        &mut display_buffer,
    );
    display.clear(Rgb565::BLACK).unwrap();

    let mut event_sub = EVENT_CHANNEL.subscriber().unwrap();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .build();
    let character_style_red = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::RED)
        .build();
    let character_style_orange = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::YELLOW)
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Middle)
        .build();

    let centre = display.bounding_box().center();
    let width = display.bounding_box().size.width;

    TextBox::with_textbox_style(
        "Buttons",
        Rectangle::with_center(centre - Point::new(0, 80), Size::new(width, 50)),
        character_style,
        textbox_style,
    )
    .draw(&mut display)
    .unwrap();

    TextBox::with_textbox_style(
        "Hexpansions",
        Rectangle::with_center(centre + Point::new(0, 80), Size::new(width, 50)),
        character_style,
        textbox_style,
    )
    .draw(&mut display)
    .unwrap();

    loop {
        match event_sub.next_message().await {
            WaitResult::Lagged(_) => panic!(),
            WaitResult::Message(Event::Button(event)) => {
                let (text, x) = match event.button() {
                    tildagon::front::emf2024::SystemButton::A => ("A", -50),
                    tildagon::front::emf2024::SystemButton::B => ("B", -30),
                    tildagon::front::emf2024::SystemButton::C => ("C", -10),
                    tildagon::front::emf2024::SystemButton::D => ("D", 10),
                    tildagon::front::emf2024::SystemButton::E => ("E", 30),
                    tildagon::front::emf2024::SystemButton::F => ("F", 50),
                };

                TextBox::with_textbox_style(
                    text,
                    Rectangle::with_center(centre + Point::new(x, -60), Size::new(width, 50)),
                    match event.now().state() {
                        ButtonState::Pressed => character_style_orange,
                        ButtonState::Released => character_style,
                    },
                    textbox_style,
                )
                .draw(&mut display)
                .unwrap();
            }
            WaitResult::Message(Event::HexpansionSlot(event)) => {
                let (text, x) = match event.slot() {
                    HexpansionSlot::A => ("A", -50),
                    HexpansionSlot::B => ("B", -30),
                    HexpansionSlot::C => ("C", -10),
                    HexpansionSlot::D => ("D", 10),
                    HexpansionSlot::E => ("E", 30),
                    HexpansionSlot::F => ("F", 50),
                };

                TextBox::with_textbox_style(
                    text,
                    Rectangle::with_center(centre + Point::new(x, 60), Size::new(width, 50)),
                    match event.state() {
                        HexpansionState::Disabled => character_style_red,
                        HexpansionState::Empty => character_style_orange,
                        HexpansionState::Occupied => character_style,
                    },
                    textbox_style,
                )
                .draw(&mut display)
                .unwrap();
            }
        }
    }
}

#[embassy_executor::task]
async fn led_task(
    r: LedResources<'static>,
    rmt_channel: esp_hal::rmt::ChannelCreator<'static, esp_hal::Blocking, 0>,
) {
    let mut rmt_buffer = [esp_hal::rmt::PulseCode::end_marker();
        tildagon::front::Emf2024FrontBoard::RMT_BUFFER_SIZE];

    let mut adapter =
        tildagon::esp_hal_smartled::SmartLedsAdapter::new(rmt_channel, r.data, &mut rmt_buffer);

    let mut leds = <tildagon::front::Emf2024FrontBoard as FrontBoardLeds>::PixelBuffer::default();

    const HEX_DISABLED_COLOUR: RGB8 = RGB8::new(255, 0, 0);
    const HEX_EMPTY_COLOUR: RGB8 = RGB8::new(255, 192, 0);
    const HEX_OCCUPIED_COLOUR: RGB8 = RGB8::new(255, 255, 255);

    *leds.base_board() = RGB8::new(128, 0, 128);

    adapter.write(leds.into_iter()).unwrap();

    let mut colour = Hsv {
        hue: 0,
        sat: 255,
        val: 127,
    };

    let mut front_pixel_tick = Ticker::every(Duration::from_millis(50));
    let mut event_sub = EVENT_CHANNEL.subscriber().unwrap();

    loop {
        match select(event_sub.next_message(), front_pixel_tick.next()).await {
            Either::First(WaitResult::Lagged(_)) => panic!(),
            Either::First(WaitResult::Message(Event::HexpansionSlot(event))) => {
                *leds.hexpansion_port(*event.slot()) = match *event.state() {
                    HexpansionState::Disabled => HEX_DISABLED_COLOUR,
                    HexpansionState::Empty => HEX_EMPTY_COLOUR,
                    HexpansionState::Occupied => HEX_OCCUPIED_COLOUR,
                };

                adapter.write(leds.into_iter()).unwrap();
            }
            Either::First(_) => {}
            Either::Second(_) => {
                colour.hue = colour.hue.wrapping_add(2);
                leds.front().iter_mut().for_each(|d| *d = hsv2rgb(colour));

                adapter.write(leds.into_iter()).unwrap();
            }
        }
    }
}

#[embassy_executor::task]
async fn button_logic_task() {
    let mut event_sub = EVENT_CHANNEL.subscriber().unwrap();
    let hex_control_pub = HEX_CONTROL_CHANNEL.publisher().unwrap();

    loop {
        match event_sub.next_message().await {
            WaitResult::Lagged(_) => panic!(),
            WaitResult::Message(Event::Button(event)) => {
                if event.released()
                    && let Some(short_press) = event.duration().map(|d| d < Duration::from_secs(1))
                {
                    let slot = match event.button() {
                        tildagon::front::emf2024::SystemButton::A => HexpansionSlot::A,
                        tildagon::front::emf2024::SystemButton::B => HexpansionSlot::B,
                        tildagon::front::emf2024::SystemButton::C => HexpansionSlot::C,
                        tildagon::front::emf2024::SystemButton::D => HexpansionSlot::D,
                        tildagon::front::emf2024::SystemButton::E => HexpansionSlot::E,
                        tildagon::front::emf2024::SystemButton::F => HexpansionSlot::F,
                    };

                    let msg = HexpansionControlMsg {
                        slot,
                        enable: short_press,
                    };

                    hex_control_pub.publish(msg).await;
                }
            }
            WaitResult::Message(_) => {}
        }
    }
}
