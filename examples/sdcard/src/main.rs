#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

mod exclusive_device;
mod fake_pin;

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_rtt_target as _;
use static_cell::StaticCell;
use tildagon::{
    buttons::Buttons,
    esp_hal::{
        self,
        clock::CpuClock,
        dma::{DmaRxBuf, DmaTxBuf},
        dma_buffers,
        gpio::{Level, Output},
        rmt::Rmt,
        spi::{
            Mode,
            master::{Config, Spi},
        },
        time::Rate,
        timer::timg::TimerGroup,
    },
    hexpansion_slots::{HexpansionSlot, HexpansionSlotControl},
    i2c::{SharedI2cBus, SharedI2cDevice},
    leds::Leds,
    pins::{
        PinControl,
        async_digital::{InputPin, OutputPin},
    },
    resources::*,
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

    let _buttons = Buttons::try_new(SharedI2cDevice::new(i2c_system), pins.button)
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

    let hex_a_fast = r.hexpansion_a;
    let hex_a_slow = pins.hexpansion_a;

    hex_slots
        .set_enabled(HexpansionSlot::A, true)
        .await
        .unwrap();

    let mut card_detect_1 = hex_a_slow
        .ls_1
        .into_input(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();
    let mut card_detect_2 = hex_a_slow
        .ls_4
        .into_input(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();

    let cs_2 = hex_a_slow
        .ls_5
        .into_output(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();

    info!("Card 1 detect: {}", card_detect_1.is_low().await);
    info!("Card 2 detect: {}", card_detect_2.is_low().await);

    let dma_channel = p.DMA_CH1;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi = Spi::new(
        p.SPI3,
        Config::default()
            .with_frequency(Rate::from_mhz(16))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(hex_a_fast.hs_3)
    .with_mosi(hex_a_fast.hs_2)
    .with_miso(hex_a_fast.hs_4)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf);

    let cs = Output::new(hex_a_fast.hs_1, Level::High, Default::default());

    let dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();

    let sdcard = embedded_sdmmc::SdCard::new(dev, Delay);
    info!("Card type {}", sdcard.get_card_type());
    info!("Card size is {} bytes", sdcard.num_bytes().unwrap());
}
