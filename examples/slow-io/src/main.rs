#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_hal::pwm::SetDutyCycle;
use panic_rtt_target as _;
use static_cell::StaticCell;
use tildagon::{
    embedded_aw9523::PinConfiguration,
    esp_hal::{self, clock::CpuClock, timer::timg::TimerGroup},
    hexpansions::{HexpansionPort, HexpansionPortControl},
    i2c::SharedI2cBus,
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

    let mut hex_slots = HexpansionPortControl::new(pins.hexpansion_detect)
        .await
        .unwrap();

    // A little time for other tasks to start.
    // Hacky as all fuck but good enough for a demo.
    // Use channels to indicate readiness properly, mkay.
    Timer::after_millis(500).await;

    hex_slots
        .set_enabled(HexpansionPort::A, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionPort::B, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionPort::C, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionPort::D, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionPort::E, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionPort::F, true)
        .await
        .unwrap();

    let hex_a_pins = pins.hexpansion_a;

    let mut a1 = hex_a_pins.ls_1.try_into_led().await.unwrap();
    let mut a2 = hex_a_pins.ls_2.try_into_led().await.unwrap();
    let mut a3 = hex_a_pins.ls_3.try_into_led().await.unwrap();
    let mut a4 = hex_a_pins.ls_4.try_into_led().await.unwrap();
    let mut a5 = hex_a_pins.ls_5.try_into_led().await.unwrap();

    loop {
        ramp(&mut a1).await;
        ramp(&mut a2).await;
        ramp(&mut a3).await;
        ramp(&mut a4).await;
        ramp(&mut a5).await;
    }
}

async fn ramp<T: SetDutyCycle>(io: &mut T) {
    for pct in 0..100 {
        io.set_duty_cycle(pct).unwrap();
        Timer::after_millis(10).await;
    }
    io.set_duty_cycle(0).unwrap();
}
