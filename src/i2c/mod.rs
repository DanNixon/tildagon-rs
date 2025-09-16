mod tca9548a;

use crate::resources::I2cResources;
use embassy_sync::mutex::Mutex;
use esp_hal::{
    Async,
    gpio::{Level, Output},
    i2c::master::Config,
    time::Rate,
};

pub type I2c = esp_hal::i2c::master::I2c<'static, Async>;

// TODO: should this be configurable via a feature?
pub type SharingRawMutex = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub type SharedI2cBus<BUS> = Mutex<SharingRawMutex, BUS>;
pub type SharedI2cDevice<BUS> =
    embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'static, SharingRawMutex, BUS>;

pub async fn i2c_bus(r: I2cResources<'static>) -> (SharedI2cBus<I2c>, Output<'static>) {
    let config = Config::default().with_frequency(Rate::from_khz(100));

    defmt::info!("conf: {}", config);

    let i2c = esp_hal::i2c::master::I2c::new(r.i2c, config)
        .unwrap()
        .with_sda(r.sda)
        .with_scl(r.scl)
        .into_async();

    let mut reset = Output::new(r.reset, Level::High, Default::default());
    reset.set_low();
    embassy_time::Timer::after_millis(10).await;
    reset.set_high();

    (SharedI2cBus::new(i2c), reset)
}

macro_rules! define_i2c_bus {
    ($fn_name:ident, $bus_name:ident, $bus:ident) => {
        pub type $bus_name =
            $crate::i2c::tca9548a::Bus<I2c, { $crate::i2c::tca9548a::BusNumber::$bus }>;

        pub fn $fn_name(
            i2c: &'static $crate::i2c::SharedI2cBus<I2c>,
        ) -> $crate::i2c::SharedI2cBus<$crate::i2c::$bus_name> {
            $crate::i2c::SharedI2cBus::new($crate::i2c::tca9548a::Bus::new(&i2c))
        }
    };
}

define_i2c_bus!(system_i2c_bus, SystemI2cBus, Bus7);
define_i2c_bus!(top_i2c_bus, TopBoardI2cBus, Bus0);
define_i2c_bus!(hexpansion_a_i2c_bus, HexpansionAI2cBus, Bus1);
define_i2c_bus!(hexpansion_b_i2c_bus, HexpansionBI2cBus, Bus2);
define_i2c_bus!(hexpansion_c_i2c_bus, HexpansionCI2cBus, Bus3);
define_i2c_bus!(hexpansion_d_i2c_bus, HexpansionDI2cBus, Bus4);
define_i2c_bus!(hexpansion_e_i2c_bus, HexpansionEI2cBus, Bus5);
define_i2c_bus!(hexpansion_f_i2c_bus, HexpansionFI2cBus, Bus6);
