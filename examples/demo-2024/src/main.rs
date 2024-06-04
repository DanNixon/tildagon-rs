#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::RefCell;
use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::{Duration, Ticker, Timer};
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::{Dimensions, Point, RgbColor, Size},
    primitives::Rectangle,
};
use embedded_hal::digital::OutputPin;
use embedded_text::{
    TextBox,
    alignment::{HorizontalAlignment, VerticalAlignment},
    style::TextBoxStyleBuilder,
};
use panic_rtt_target as _;
use smart_leds::{
    RGB8,
    hsv::{Hsv, hsv2rgb},
};
use static_cell::StaticCell;
use tildagon::{
    buttons::{Button, ButtonEvent, ButtonState, Buttons},
    esp_hal::{self, clock::CpuClock, rmt::Rmt, time::Rate, timer::systimer::SystemTimer},
    hexpansion_slots::{
        HexpansionSlot, HexpansionSlotControl, HexpansionSlotEvent, HexpansionState,
    },
    i2c::{SharedI2cDevice, SystemI2cBus},
    leds::Leds,
    pins::PinControl,
    resources::*,
};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let config = tildagon::esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = tildagon::esp_hal::init(config);
    let r = tildagon::split_resources!(p);

    esp_alloc::heap_allocator!(size: 64 * 1024);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024);

    let timer0 = SystemTimer::new(p.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    static I2C_BUS: StaticCell<RefCell<tildagon::i2c::I2c>> = StaticCell::new();
    let (bus, _reset) = tildagon::i2c::i2c_bus(r.i2c).await;
    let i2c_bus = I2C_BUS.init(bus);

    static I2C_SYSTEM: StaticCell<RefCell<tildagon::i2c::SystemI2cBus>> = StaticCell::new();
    let i2c_system = I2C_SYSTEM.init(tildagon::i2c::system_i2c_bus(i2c_bus));

    let mut pin_control = PinControl::new(i2c_system);
    // pin_control.reset().unwrap();
    pin_control.init().unwrap();
    let pins = pin_control.pins();

    let mut usb_sel = pins
        .other
        .usb_select
        .into_output(SharedI2cDevice::new(i2c_system))
        .unwrap();
    usb_sel.set_low().unwrap();

    let mut buttons = Buttons::try_new(SharedI2cDevice::new(i2c_system), pins.button).unwrap();

    let mut hex_slots =
        HexpansionSlotControl::try_new(SharedI2cDevice::new(i2c_system), pins.hexpansion_detect)
            .unwrap();

    let rmt: Rmt<'_, esp_hal::Async> = Rmt::new(p.RMT, Rate::from_mhz(80)).unwrap().into_async();

    let mut leds = Leds::try_new(
        SharedI2cDevice::new(i2c_system),
        pins.led,
        r.led,
        rmt.channel0,
    )
    .unwrap();
    leds.set_power(true).unwrap();

    spawner.must_spawn(display_task(r.top_board, r.display));
    spawner.must_spawn(led_task(leds));
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
                let regs = pin_control.read_input_registers().unwrap();

                for event in buttons.update(&regs) {
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
                hex_slots.set_enabled(msg.slot, msg.enable).unwrap();
            }
        }
    }
}

#[derive(Clone)]
enum Event {
    Button(ButtonEvent),
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

#[embassy_executor::task]
async fn display_task(top_board: TopBoardResources<'static>, display: DisplayResources<'static>) {
    let mut display_buffer = [0_u8; 512];
    let mut display = tildagon::display::init(top_board, display, &mut display_buffer);
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
                    Button::A => ("A", -50),
                    Button::B => ("B", -30),
                    Button::C => ("C", -10),
                    Button::D => ("D", 10),
                    Button::E => ("E", 30),
                    Button::F => ("F", 50),
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
async fn led_task(mut leds: Leds<SharedI2cDevice<SystemI2cBus>>) {
    const HEX_DISABLED_COLOUR: RGB8 = RGB8::new(255, 0, 0);
    const HEX_EMPTY_COLOUR: RGB8 = RGB8::new(255, 192, 0);
    const HEX_OCCUPIED_COLOUR: RGB8 = RGB8::new(255, 255, 255);

    *leds.main_board_pixel() = RGB8::new(128, 0, 128);

    leds.write().await.unwrap();

    let mut colour = Hsv {
        hue: 0,
        sat: 255,
        val: 127,
    };

    let mut front_pixel_tick = Ticker::every(Duration::from_millis(100));
    let mut event_sub = EVENT_CHANNEL.subscriber().unwrap();

    loop {
        match select(event_sub.next_message(), front_pixel_tick.next()).await {
            Either::First(WaitResult::Lagged(_)) => panic!(),
            Either::First(WaitResult::Message(Event::HexpansionSlot(event))) => {
                *leds.hexpansion_pixel(*event.slot()) = match *event.state() {
                    HexpansionState::Disabled => HEX_DISABLED_COLOUR,
                    HexpansionState::Empty => HEX_EMPTY_COLOUR,
                    HexpansionState::Occupied => HEX_OCCUPIED_COLOUR,
                };

                leds.write().await.unwrap();
            }
            Either::First(_) => {}
            Either::Second(_) => {
                colour.hue = colour.hue.wrapping_add(2);

                leds.front_pixels()
                    .iter_mut()
                    .for_each(|d| *d = hsv2rgb(colour));

                leds.write().await.unwrap();
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
                if event.released() {
                    if let Some(short_press) = event.duration().map(|d| d < Duration::from_secs(1))
                    {
                        let slot = match event.button() {
                            Button::A => HexpansionSlot::A,
                            Button::B => HexpansionSlot::B,
                            Button::C => HexpansionSlot::C,
                            Button::D => HexpansionSlot::D,
                            Button::E => HexpansionSlot::E,
                            Button::F => HexpansionSlot::F,
                        };

                        let msg = HexpansionControlMsg {
                            slot,
                            enable: short_press,
                        };

                        hex_control_pub.publish(msg).await;
                    }
                }
            }
            WaitResult::Message(_) => {}
        }
    }
}
