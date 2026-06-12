use crate::{
    i2c::{SharedI2cBus, SharedI2cDevice, SystemI2cBus},
    resources::SystemResources,
};
use bq25895::Bq25895;
use defmt::info;
use esp_hal::gpio::Input;

pub struct SystemInterrupt {
    int: Input<'static>,
}

impl SystemInterrupt {
    pub fn new(r: SystemResources<'static>) -> Self {
        let int = Input::new(r.int, Default::default());
        Self { int }
    }

    pub async fn wait_for_interrupt(&mut self) {
        self.int.wait_for_falling_edge().await;
        info!("System interrupt trigger");
    }
}

pub fn new_bq25895(
    i2c_system: &'static SharedI2cBus<SystemI2cBus>,
) -> Bq25895<bq25895::Interface<SharedI2cDevice<SystemI2cBus>>> {
    let bq_interface = bq25895::Interface::new(SharedI2cDevice::new(i2c_system));
    Bq25895::new(bq_interface)
}
