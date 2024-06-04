use super::{
    aw9523b::{GpioDirection, PinMode, set_io_direction, set_io_state, set_pin_mode},
    input::InputPin,
    pin::TypeErasedPin,
};
use embedded_hal::{
    digital::{ErrorKind, PinState},
    i2c::I2c,
};

pub struct OutputPin<I2C> {
    bus: I2C,
    pin: TypeErasedPin,
}

impl<I2C, E> OutputPin<I2C>
where
    I2C: I2c<Error = E>,
{
    pub(crate) fn try_new(mut bus: I2C, pin: TypeErasedPin) -> Result<Self, E> {
        set_pin_mode(&mut bus, &pin, PinMode::Gpio)?;
        set_io_direction(&mut bus, &pin, GpioDirection::Output)?;
        Ok(Self { bus, pin })
    }

    pub fn into_input(self) -> Result<InputPin<I2C>, E> {
        InputPin::try_new(self.bus, self.pin)
    }
}

impl<I2C> embedded_hal::digital::ErrorType for OutputPin<I2C> {
    type Error = ErrorKind;
}

impl<I2C, E> embedded_hal::digital::OutputPin for OutputPin<I2C>
where
    I2C: I2c<Error = E>,
{
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_state(PinState::High)
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_state(PinState::Low)
    }

    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        set_io_state(&mut self.bus, &self.pin, state).map_err(|_| ErrorKind::Other)
    }
}
