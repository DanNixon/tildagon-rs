use super::{
    aw9523b::{PinMode, Port, Register, set_pin_mode, write_register},
    pin::{PinExt, TypeErasedPin},
};
use defmt::debug;
use embedded_hal::pwm::ErrorKind;

pub struct LedPin<I2C> {
    bus: I2C,
    pin: TypeErasedPin,
}

impl<I2C, E> LedPin<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    pub(crate) async fn try_new(mut bus: I2C, pin: TypeErasedPin) -> Result<Self, E> {
        set_pin_mode(&mut bus, &pin, PinMode::Led).await?;
        Ok(Self { bus, pin })
    }
}

impl<I2C> embedded_hal::pwm::ErrorType for LedPin<I2C> {
    type Error = ErrorKind;
}

impl<I2C, E> super::async_pwm::SetDutyCycle for LedPin<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    async fn max_duty_cycle(&self) -> u16 {
        255
    }

    async fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        let register = match (self.pin.port(), self.pin.pin()) {
            (Port::Port0, 0) => Register::DIM4_P00,
            (Port::Port0, 1) => Register::DIM5_P01,
            (Port::Port0, 2) => Register::DIM6_P02,
            (Port::Port0, 3) => Register::DIM7_P03,
            (Port::Port0, 4) => Register::DIM8_P04,
            (Port::Port0, 5) => Register::DIM9_P05,
            (Port::Port0, 6) => Register::DIM10_P06,
            (Port::Port0, 7) => Register::DIM11_P07,
            (Port::Port1, 0) => Register::DIM0_P10,
            (Port::Port1, 1) => Register::DIM1_P11,
            (Port::Port1, 2) => Register::DIM2_P12,
            (Port::Port1, 3) => Register::DIM3_P13,
            (Port::Port1, 4) => Register::DIM12_P14,
            (Port::Port1, 5) => Register::DIM13_P15,
            (Port::Port1, 6) => Register::DIM14_P16,
            (Port::Port1, 7) => Register::DIM15_P17,
            _ => unreachable!(),
        };

        write_register(&mut self.bus, self.pin.address(), register, duty as u8)
            .await
            .map_err(|_| ErrorKind::Other)?;

        debug!("Set pin {} to {}", self.pin, duty);
        Ok(())
    }
}
