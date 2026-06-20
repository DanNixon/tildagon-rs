use crate::{
    i2c::{SharedI2cBus, SharedI2cDevice, SystemI2cBus},
    pins::LedPins,
    resources::SystemResources,
};
use bq25895::Bq25895;
use defmt::info;
use embedded_aw9523::{Output, async_traits::digital::OutputPin};
use embedded_hal::digital::PinState;
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

pub struct OnboardLedPower<I2C> {
    pwr: Output<I2C>,
}

impl<I2C> OnboardLedPower<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    pub fn new(r: LedPins<I2C>) -> Self {
        Self {
            pwr: r.power_enable,
        }
    }

    pub async fn set_on(
        &mut self,
        on: bool,
    ) -> Result<(), <Output<I2C> as embedded_hal::digital::ErrorType>::Error> {
        self.pwr
            .set_state(match on {
                true => PinState::High,
                false => PinState::Low,
            })
            .await
    }
}

pub fn new_bq25895(
    i2c_system: &'static SharedI2cBus<SystemI2cBus>,
) -> Bq25895<bq25895::Interface<SharedI2cDevice<SystemI2cBus>>> {
    let bq_interface = bq25895::Interface::new(SharedI2cDevice::new(i2c_system));
    Bq25895::new(bq_interface)
}
