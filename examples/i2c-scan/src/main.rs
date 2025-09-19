#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use panic_rtt_target as _;
use static_cell::StaticCell;
use tildagon::{
    esp_hal::{self, clock::CpuClock, timer::systimer::SystemTimer},
    i2c::{
        HexpansionAI2cBus, HexpansionBI2cBus, HexpansionCI2cBus, HexpansionDI2cBus,
        HexpansionEI2cBus, HexpansionFI2cBus, SharedI2cBus, SystemI2cBus, TopBoardI2cBus,
    },
    resources::*,
};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

macro_rules! scan_and_report {
    ($bus:expr, $name:expr) => {
        info!("BUS: {}", $name);
        let results = tildagon::i2c::scan::scan_bus(&$bus).await;
        tildagon::i2c::scan::report_present_devices(&results);
        // tildagon::i2c::scan::report_absent_devices(&results);
    };
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
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

    static I2C_SYSTEM: StaticCell<SharedI2cBus<SystemI2cBus>> = StaticCell::new();
    let i2c_system = I2C_SYSTEM.init(tildagon::i2c::system_i2c_bus(i2c_bus));
    static I2C_TOP: StaticCell<SharedI2cBus<TopBoardI2cBus>> = StaticCell::new();
    let i2c_top = I2C_TOP.init(tildagon::i2c::top_i2c_bus(i2c_bus));
    static I2C_HEX_A: StaticCell<SharedI2cBus<HexpansionAI2cBus>> = StaticCell::new();
    let i2c_hex_a = I2C_HEX_A.init(tildagon::i2c::hexpansion_a_i2c_bus(i2c_bus));
    static I2C_HEX_B: StaticCell<SharedI2cBus<HexpansionBI2cBus>> = StaticCell::new();
    let i2c_hex_b = I2C_HEX_B.init(tildagon::i2c::hexpansion_b_i2c_bus(i2c_bus));
    static I2C_HEX_C: StaticCell<SharedI2cBus<HexpansionCI2cBus>> = StaticCell::new();
    let i2c_hex_c = I2C_HEX_C.init(tildagon::i2c::hexpansion_c_i2c_bus(i2c_bus));
    static I2C_HEX_D: StaticCell<SharedI2cBus<HexpansionDI2cBus>> = StaticCell::new();
    let i2c_hex_d = I2C_HEX_D.init(tildagon::i2c::hexpansion_d_i2c_bus(i2c_bus));
    static I2C_HEX_E: StaticCell<SharedI2cBus<HexpansionEI2cBus>> = StaticCell::new();
    let i2c_hex_e = I2C_HEX_E.init(tildagon::i2c::hexpansion_e_i2c_bus(i2c_bus));
    static I2C_HEX_F: StaticCell<SharedI2cBus<HexpansionFI2cBus>> = StaticCell::new();
    let i2c_hex_f = I2C_HEX_F.init(tildagon::i2c::hexpansion_f_i2c_bus(i2c_bus));

    let mut tick = Ticker::every(Duration::from_secs(30));

    loop {
        scan_and_report!(i2c_system, "System");
        scan_and_report!(i2c_top, "Top Board");
        scan_and_report!(i2c_hex_a, "Hexpansion A");
        scan_and_report!(i2c_hex_b, "Hexpansion B");
        scan_and_report!(i2c_hex_c, "Hexpansion C");
        scan_and_report!(i2c_hex_d, "Hexpansion D");
        scan_and_report!(i2c_hex_e, "Hexpansion E");
        scan_and_report!(i2c_hex_f, "Hexpansion F");

        tick.next().await;
    }
}
