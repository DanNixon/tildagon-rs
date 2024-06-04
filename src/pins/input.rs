use super::{
    aw9523b::{
        GpioDirection, PinMode, Port, Register, read_register, set_io_direction, set_pin_mode,
        write_register,
    },
    output::OutputPin,
    pin::{PinExt, TypeErasedPin},
};
use defmt::debug;
use embedded_hal::{digital::ErrorKind, i2c::I2c};

pub struct InputPin<I2C> {
    bus: I2C,
    pin: TypeErasedPin,
}

impl<I2C, E> InputPin<I2C>
where
    I2C: I2c<Error = E>,
{
    pub(crate) fn try_new(mut bus: I2C, pin: TypeErasedPin) -> Result<Self, E> {
        set_pin_mode(&mut bus, &pin, PinMode::Gpio)?;
        set_io_direction(&mut bus, &pin, GpioDirection::Input)?;
        Ok(Self { bus, pin })
    }

    pub fn into_output(self) -> Result<OutputPin<I2C>, E> {
        OutputPin::try_new(self.bus, self.pin)
    }

    pub fn set_interrupt(&mut self, enable: bool) -> Result<(), E> {
        let register = match self.pin.port() {
            Port::Port0 => Register::INT_P0,
            Port::Port1 => Register::INT_P1,
        };

        let value = read_register(&mut self.bus, self.pin.address(), register)?;

        let value = match enable {
            true => value & !self.pin.bit(),
            false => value | self.pin.bit(),
        };

        write_register(&mut self.bus, self.pin.address(), register, value)?;

        debug!("Set interrupt enable for pin {} to {}", self.pin, enable);
        Ok(())
    }
}

impl<I2C> embedded_hal::digital::ErrorType for InputPin<I2C> {
    type Error = ErrorKind;
}

impl<I2C, E> embedded_hal::digital::InputPin for InputPin<I2C>
where
    I2C: I2c<Error = E>,
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_low()?)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        let register = match self.pin.port() {
            Port::Port0 => Register::INPUT_P0,
            Port::Port1 => Register::INPUT_P1,
        };

        let value = read_register(&mut self.bus, self.pin.address(), register)
            .map_err(|_| ErrorKind::Other)?;

        let is_low = value & self.pin.bit() == 0;

        debug!("Pin {} is low? {}", self.pin, is_low);
        Ok(is_low)
    }
}
