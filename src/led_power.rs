use crate::pins::LedPins;
use embedded_aw9523::{Output, async_traits::digital::OutputPin};
use embedded_hal::digital::PinState;

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

    pub async fn set(
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
