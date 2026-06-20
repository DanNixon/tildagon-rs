use crate::i2c::{SharedI2cBus, SharedI2cDevice, SystemI2cBus};
use bq25895::{Bq25895, Interface};

pub fn new_bq25895(
    i2c_system: &'static SharedI2cBus<SystemI2cBus>,
) -> Bq25895<Interface<SharedI2cDevice<SystemI2cBus>>> {
    let bq_interface = Interface::new(SharedI2cDevice::new(i2c_system));
    Bq25895::new(bq_interface)
}
