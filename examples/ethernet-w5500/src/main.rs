#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};
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
use smart_leds::{
    RGB8,
    hsv::{Hsv, hsv2rgb},
};
use static_cell::StaticCell;
use tildagon::{
    buttons::Buttons,
    esp_hal::{
        self,
        clock::CpuClock,
        dma::{DmaRxBuf, DmaTxBuf},
        dma_buffers,
        gpio::Input,
        interrupt::software::SoftwareInterruptControl,
        rmt::Rmt,
        spi::{
            Mode,
            master::{Config, Spi},
        },
        system::Stack,
        time::Rate,
        timer::timg::TimerGroup,
    },
    hexpansion_slots::{HexpansionSlot, HexpansionSlotControl},
    i2c::{SharedI2cBus, SharedI2cDevice, SystemI2cBus},
    leds::Leds,
    pins::{PinControl, async_digital::OutputPin},
    resources::*,
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

    let mut hex_slots =
        HexpansionSlotControl::try_new(SharedI2cDevice::new(i2c_system), pins.hexpansion_detect)
            .await
            .unwrap();

    let rmt: Rmt<'_, esp_hal::Blocking> = Rmt::new(p.RMT, Rate::from_mhz(80)).unwrap();

    static RMT_BUFFER: StaticCell<tildagon::leds::RmtBuffer> = StaticCell::new();
    let rmt_buffer = RMT_BUFFER.init(tildagon::leds::make_rmt_buffer());

    let mut leds = Leds::try_new(
        SharedI2cDevice::new(i2c_system),
        pins.led,
        r.led,
        rmt.channel0,
        rmt_buffer,
    )
    .await
    .unwrap();
    leds.set_power(true).await.unwrap();

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
                spawner.must_spawn(display_task(r.top_board, r.display));
            });
        },
    );

    spawner.must_spawn(led_task(leds));

    // A little time for other tasks to start.
    // Hacky as all fuck but good enough for a demo.
    // Use channels to indicate readiness properly, mkay.
    Timer::after_millis(500).await;

    hex_slots
        .set_enabled(HexpansionSlot::A, true)
        .await
        .unwrap();

    let hex_a_fast = r.hexpansion_a;
    let hex_a_slow = pins.hexpansion_a;

    let dma_channel = p.DMA_CH1;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // Keep chip select low so that W5500 is always enabled, it is the only device on the bus anyway
    let mut cs = hex_a_slow
        .ls_1
        .into_output(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();
    cs.set_low().await.unwrap();

    let mut spi = Spi::new(
        p.SPI3,
        Config::default()
            .with_frequency(Rate::from_mhz(50))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(hex_a_fast.hs_1)
    .with_mosi(hex_a_fast.hs_2)
    .with_miso(hex_a_fast.hs_3)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    let w5500_int = Input::new(hex_a_fast.hs_4, Default::default());

    let w5500_reset = hex_a_slow
        .ls_2
        .into_output(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();

    // TODO: just mock the reset pin and assert it manually before creating the device
    let (device, runner) =
        embassy_net_wiznet::new(mac_addr, state, spi_dev, w5500_int, w5500_reset).unwrap();

    let mut tick = Ticker::every(Duration::from_secs(60));

    loop {
        // TODO
        tick.next().await;
    }
}

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
        .alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Middle)
        .build();

    let centre = display.bounding_box().center();
    let width = display.bounding_box().size.width;

    loop {
        // TODO
        Timer::after_secs(30).await;
    }
}

#[embassy_executor::task]
async fn led_task(mut leds: Leds<'static, SharedI2cDevice<SystemI2cBus>>) {
    *leds.main_board_pixel() = RGB8::new(128, 0, 128);

    leds.write().unwrap();

    let mut colour = Hsv {
        hue: 0,
        sat: 255,
        val: 127,
    };

    loop {
        // TODO
        Timer::after_secs(30).await;
    }
}
