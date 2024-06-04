mod tca9548a;

use crate::resources::I2cResources;
use core::cell::RefCell;
use esp_hal::{
    Async,
    gpio::{Level, Output},
    i2c::master::Config,
    time::Rate,
};

pub type I2c = esp_hal::i2c::master::I2c<'static, Async>;

pub async fn i2c_bus(r: I2cResources<'static>) -> (RefCell<I2c>, Output<'static>) {
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

    (RefCell::new(i2c), reset)
}

macro_rules! define_i2c_bus {
    ($fn_name:ident, $bus_name:ident, $bus:ident) => {
        pub type $bus_name =
            $crate::i2c::tca9548a::Bus<I2c, { $crate::i2c::tca9548a::BusNumber::$bus }>;

        pub fn $fn_name(
            i2c: &'static core::cell::RefCell<I2c>,
        ) -> core::cell::RefCell<$crate::i2c::$bus_name> {
            core::cell::RefCell::new($crate::i2c::tca9548a::Bus::new(&i2c))
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

pub struct SharedI2cDevice<T: 'static> {
    bus: &'static RefCell<T>,
}

impl<T> SharedI2cDevice<T> {
    #[inline]
    pub fn new(bus: &'static RefCell<T>) -> Self {
        Self { bus }
    }
}

impl<T> embedded_hal::i2c::ErrorType for SharedI2cDevice<T>
where
    T: embedded_hal::i2c::I2c,
{
    type Error = T::Error;
}

impl<T> embedded_hal::i2c::I2c for SharedI2cDevice<T>
where
    T: embedded_hal::i2c::I2c,
{
    #[inline]
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.borrow_mut();
        bus.read(address, read)
    }

    #[inline]
    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.borrow_mut();
        bus.write(address, write)
    }

    #[inline]
    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.borrow_mut();
        bus.write_read(address, write, read)
    }

    #[inline]
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let bus = &mut *self.bus.borrow_mut();
        bus.transaction(address, operations)
    }
}

// impl<T> embedded_hal_async::i2c::I2c for SharedI2cDevice<T>
// where
//     T: embedded_hal_async::i2c::I2c + embedded_hal::i2c::I2c,
// {
//     #[inline]
//     async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
//         let bus = &mut *self.bus.borrow_mut();
//         embedded_hal_async::i2c::I2c::read(bus, address, read).await
//     }

//     #[inline]
//     async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
//         let bus = &mut *self.bus.borrow_mut();
//         embedded_hal_async::i2c::I2c::write(bus, address, write).await
//     }

//     #[inline]
//     async fn write_read(
//         &mut self,
//         address: u8,
//         write: &[u8],
//         read: &mut [u8],
//     ) -> Result<(), Self::Error> {
//         let bus = &mut *self.bus.borrow_mut();
//         embedded_hal_async::i2c::I2c::write_read(bus, address, write, read).await
//     }

//     #[inline]
//     async fn transaction(
//         &mut self,
//         address: u8,
//         operations: &mut [embedded_hal::i2c::Operation<'_>],
//     ) -> Result<(), Self::Error> {
//         let bus = &mut *self.bus.borrow_mut();
//         embedded_hal_async::i2c::I2c::transaction(bus, address, operations).await
//     }
// }
