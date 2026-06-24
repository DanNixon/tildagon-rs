#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::fmt::Write;
use embassy_executor::Spawner;
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
use esp_rtos::embassy::Executor;
use heapless::String;
use panic_rtt_target as _;
use static_cell::StaticCell;
use tildagon::{
    bmi2::types::{Data, PwrCtrl},
    esp_hal::{
        self,
        clock::CpuClock,
        interrupt::software::SoftwareInterruptControl,
        peripherals::{DMA_CH0, SPI2},
        system::Stack,
        timer::timg::TimerGroup,
    },
    front::FrontBoardDisplay,
    i2c::{SharedI2cBus, SharedI2cDevice},
    pins::PinControl,
    resources::*,
    usb::{UsbPort, UsbSwitch},
};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
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

    let mut imu = tildagon::imu::init(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();
    imu.set_pwr_ctrl(PwrCtrl {
        aux_en: false,
        gyr_en: true,
        acc_en: true,
        temp_en: false,
    })
    .unwrap();

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
                spawner.must_spawn(display_task(r.front_board, p.SPI2, p.DMA_CH0));
            });
        },
    );

    // A little time for other tasks to start.
    // Hacky as all fuck but good enough for a demo.
    // Use channels to indicate readiness properly, mkay.
    Timer::after_millis(500).await;

    let mut imu_tick = Ticker::every(Duration::from_millis(250));
    let event_pub = EVENT_CHANNEL.publisher().unwrap();

    loop {
        imu_tick.next().await;

        let data: Data = imu.get_data().unwrap();
        event_pub
            .publish(Event::ImuAxisData(ImuData {
                gyro_x: data.gyr.x,
                gyro_y: data.gyr.y,
                gyro_z: data.gyr.z,
                accel_x: data.acc.x,
                accel_y: data.acc.y,
                accel_z: data.acc.z,
            }))
            .await;
    }
}

#[derive(Clone)]
enum Event {
    ImuAxisData(ImuData),
}

#[derive(Clone)]
struct ImuData {
    gyro_x: i16,
    gyro_y: i16,
    gyro_z: i16,
    accel_x: i16,
    accel_y: i16,
    accel_z: i16,
}

static EVENT_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Event, 12, 4, 4> =
    PubSubChannel::new();

#[embassy_executor::task]
async fn display_task(
    front_board: FrontBoardResources<'static>,
    spi: SPI2<'static>,
    dma: DMA_CH0<'static>,
) {
    let mut display_buffer = [0_u8; 512];
    let mut display = <tildagon::front::Emf2024FrontBoard as FrontBoardDisplay>::Display::init(
        front_board,
        spi,
        dma,
        &mut display_buffer,
    );
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
                    (data.gyro_x, box_gx),
                    (data.gyro_y, box_gy),
                    (data.gyro_z, box_gz),
                    (data.accel_x, box_ax),
                    (data.accel_y, box_ay),
                    (data.accel_z, box_az),
                ] {
                    let mut buf = String::<16>::new();
                    write!(&mut buf, "{reading:.2}").unwrap();
                    TextBox::with_textbox_style(&buf, bbox, character_style, textbox_style)
                        .draw(&mut display)
                        .unwrap();
                }
            }
        }
    }
}
