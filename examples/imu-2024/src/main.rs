#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::{fmt::Write, ptr::addr_of_mut};
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
    prelude::{Dimensions, Point, Primitive, RgbColor, Size},
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use embedded_text::{
    TextBox,
    alignment::{HorizontalAlignment, VerticalAlignment},
    style::TextBoxStyleBuilder,
};
use esp_hal_embassy::Executor;
use heapless::String;
use panic_rtt_target as _;
use smart_leds::RGB8;
use static_cell::StaticCell;
use tildagon::{
    buttons::Buttons,
    esp_hal::{
        self,
        clock::CpuClock,
        rmt::Rmt,
        system::{CpuControl, Stack},
        time::Rate,
        timer::systimer::SystemTimer,
    },
    i2c::{SharedI2cBus, SharedI2cDevice, SystemI2cBus},
    imu::bmi2::types::{Data, PwrCtrl},
    leds::Leds,
    pins::{PinControl, async_digital::OutputPin},
    resources::*,
};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

static mut APP_CORE_STACK: Stack<8192> = Stack::new();

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

    static I2C_BUS: StaticCell<SharedI2cBus<tildagon::i2c::I2c>> = StaticCell::new();
    let (bus, _reset) = tildagon::i2c::i2c_bus(r.i2c).await;
    let i2c_bus = I2C_BUS.init(bus);

    static I2C_SYSTEM: StaticCell<SharedI2cBus<tildagon::i2c::SystemI2cBus>> = StaticCell::new();
    let i2c_system = I2C_SYSTEM.init(tildagon::i2c::system_i2c_bus(i2c_bus));

    let mut pin_control = PinControl::new(i2c_system);
    // pin_control.reset().unwrap();
    pin_control.init().await.unwrap();
    let pins = pin_control.pins();

    let mut usb_sel = pins
        .other
        .usb_select
        .into_output(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();
    usb_sel.set_low().await.unwrap();

    let mut buttons = Buttons::try_new(SharedI2cDevice::new(i2c_system), pins.button)
        .await
        .unwrap();

    let rmt: Rmt<'_, esp_hal::Blocking> = Rmt::new(p.RMT, Rate::from_mhz(80)).unwrap();

    let mut leds = Leds::try_new(
        SharedI2cDevice::new(i2c_system),
        pins.led,
        r.led,
        rmt.channel0,
    )
    .await
    .unwrap();
    leds.set_power(true).await.unwrap();

    let mut imu = tildagon::imu::init(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();
    imu.set_pwr_ctrl(PwrCtrl {
        aux_en: false,
        gyr_en: true,
        acc_en: true,
        temp_en: false,
    })
    .await
    .unwrap();

    let mut cpu_control = CpuControl::new(p.CPU_CTRL);
    let _guard = cpu_control
        .start_app_core(unsafe { &mut *addr_of_mut!(APP_CORE_STACK) }, move || {
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                spawner.must_spawn(display_task(r.top_board, r.display));
            });
        })
        .unwrap();

    spawner.must_spawn(led_task(leds));

    // A little time for other tasks to start.
    // Hacky as all fuck but good enough for a demo.
    // Use channels to indicate readiness properly, mkay.
    Timer::after_millis(500).await;

    let mut io_tick = Ticker::every(Duration::from_millis(100));
    let mut imu_tick = Ticker::every(Duration::from_millis(250));
    let event_pub = EVENT_CHANNEL.publisher().unwrap();

    loop {
        match select(io_tick.next(), imu_tick.next()).await {
            Either::First(_) => {
                let regs = pin_control.read_input_registers().await.unwrap();

                for event in buttons.update(&regs) {
                    info!("Button event: {}", event);
                    event_pub.publish(Event::Button).await;
                }
            }
            Either::Second(_) => {
                let data = imu.get_data().await.unwrap();
                info!("IMU axis data: {}", data);
                event_pub.publish(Event::ImuAxisData(data)).await;
            }
        }
    }
}

#[derive(Clone)]
enum Event {
    Button,
    // Button(ButtonEvent),
    ImuAxisData(Data),
}

static EVENT_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Event, 12, 4, 4> =
    PubSubChannel::new();

#[embassy_executor::task]
async fn display_task(top_board: TopBoardResources<'static>, display: DisplayResources<'static>) {
    let mut display_buffer = [0_u8; 512];
    let mut display = tildagon::display::init(top_board, display, &mut display_buffer);
    display.clear(Rgb565::BLACK).unwrap();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .alignment(HorizontalAlignment::Right)
        .vertical_alignment(VerticalAlignment::Middle)
        .build();

    let centre = display.bounding_box().center();

    let labels = [
        ("Gx", -50),
        ("Gy", -30),
        ("Gz", -10),
        ("Ax", 10),
        ("Ay", 30),
        ("Az", 50),
    ];

    for (text, offset) in labels {
        TextBox::with_textbox_style(
            text,
            Rectangle::with_center(centre + Point::new(-40, offset), Size::new(60, 20)),
            character_style,
            textbox_style,
        )
        .draw(&mut display)
        .unwrap();
    }

    let box_gx = Rectangle::with_center(centre + Point::new(30, labels[0].1), Size::new(60, 20));
    let box_gy = Rectangle::with_center(centre + Point::new(30, labels[1].1), Size::new(60, 20));
    let box_gz = Rectangle::with_center(centre + Point::new(30, labels[2].1), Size::new(60, 20));
    let box_ax = Rectangle::with_center(centre + Point::new(30, labels[3].1), Size::new(60, 20));
    let box_ay = Rectangle::with_center(centre + Point::new(30, labels[4].1), Size::new(60, 20));
    let box_az = Rectangle::with_center(centre + Point::new(30, labels[5].1), Size::new(60, 20));

    let black_box = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::BLACK)
        .build();

    let mut event_sub = EVENT_CHANNEL.subscriber().unwrap();

    loop {
        match event_sub.next_message().await {
            WaitResult::Lagged(_) => panic!(),
            WaitResult::Message(Event::ImuAxisData(data)) => {
                for b in [box_gx, box_gy, box_gz, box_ax, box_ay, box_az] {
                    b.into_styled(black_box).draw(&mut display).unwrap();
                }

                for (reading, bbox) in [
                    (data.gyr.x, box_gx),
                    (data.gyr.y, box_gy),
                    (data.gyr.z, box_gz),
                    (data.acc.x, box_ax),
                    (data.acc.y, box_ay),
                    (data.acc.z, box_az),
                ] {
                    let mut buf = String::<16>::new();
                    write!(&mut buf, "{reading:.2}").unwrap();
                    TextBox::with_textbox_style(&buf, bbox, character_style, textbox_style)
                        .draw(&mut display)
                        .unwrap();
                }
            }
            WaitResult::Message(_) => {
                // nothing
            }
        }
    }
}

#[embassy_executor::task]
async fn led_task(mut leds: Leds<SharedI2cDevice<SystemI2cBus>>) {
    *leds.main_board_pixel() = RGB8::new(128, 0, 128);

    leds.write().unwrap();

    let mut event_sub = EVENT_CHANNEL.subscriber().unwrap();

    loop {
        match event_sub.next_message().await {
            WaitResult::Lagged(_) => panic!(),
            WaitResult::Message(_) => {
                // nothing
            }
        }
    }
}
