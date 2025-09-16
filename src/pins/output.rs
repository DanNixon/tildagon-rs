use super::{
    aw9523b::{GpioDirection, PinMode, set_io_direction, set_io_state, set_pin_mode},
    input::InputPin,
    pin::TypeErasedPin,
};
use embedded_hal::digital::{ErrorKind, PinState};

pub struct OutputPin<I2C> {
    bus: I2C,
    pin: TypeErasedPin,
}

impl<I2C, E> OutputPin<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    pub(crate) async fn try_new(mut bus: I2C, pin: TypeErasedPin) -> Result<Self, E> {
        set_pin_mode(&mut bus, &pin, PinMode::Gpio).await?;
        set_io_direction(&mut bus, &pin, GpioDirection::Output).await?;
        Ok(Self { bus, pin })
    }

    pub async fn into_input(self) -> Result<InputPin<I2C>, E> {
        InputPin::try_new(self.bus, self.pin).await
    }
}

impl<I2C> embedded_hal::digital::ErrorType for OutputPin<I2C> {
    type Error = ErrorKind;
}

impl<I2C, E> super::async_digital::OutputPin for OutputPin<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    async fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_state(PinState::High).await
    }

    async fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_state(PinState::Low).await
    }

    async fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        set_io_state(&mut self.bus, &self.pin, state)
            .await
            .map_err(|_| ErrorKind::Other)
    }
}
