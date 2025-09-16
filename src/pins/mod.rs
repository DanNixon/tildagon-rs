mod assignment;
pub(crate) mod aw9523b;
mod control;
mod input;
mod input_registers;
mod led;
mod output;
pub(crate) mod pin;

pub use self::{
    assignment::*, control::PinControl, input::InputPin, input_registers::InputRegisters,
    led::LedPin, output::OutputPin,
};

pub mod async_digital {
    use embedded_hal::digital::{ErrorType, PinState};

    #[allow(async_fn_in_trait)]
    pub trait OutputPin: ErrorType {
        async fn set_low(&mut self) -> Result<(), Self::Error>;
        async fn set_high(&mut self) -> Result<(), Self::Error>;

        #[inline]
        async fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
            match state {
                PinState::Low => self.set_low().await,
                PinState::High => self.set_high().await,
            }
        }
    }

    #[allow(async_fn_in_trait)]
    pub trait InputPin: ErrorType {
        async fn is_high(&mut self) -> Result<bool, Self::Error>;
        async fn is_low(&mut self) -> Result<bool, Self::Error>;
    }
}

pub mod async_pwm {
    use embedded_hal::pwm::ErrorType;

    #[allow(async_fn_in_trait)]
    pub trait SetDutyCycle: ErrorType {
        async fn max_duty_cycle(&self) -> u16;
        async fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error>;

        #[inline]
        async fn set_duty_cycle_fully_off(&mut self) -> Result<(), Self::Error> {
            self.set_duty_cycle(0).await
        }

        #[inline]
        async fn set_duty_cycle_fully_on(&mut self) -> Result<(), Self::Error> {
            self.set_duty_cycle(self.max_duty_cycle().await).await
        }

        #[inline]
        async fn set_duty_cycle_fraction(
            &mut self,
            num: u16,
            denom: u16,
        ) -> Result<(), Self::Error> {
            debug_assert!(denom != 0);
            debug_assert!(num <= denom);
            let duty = u32::from(num) * u32::from(self.max_duty_cycle().await) / u32::from(denom);

            // This is safe because we know that `num <= denom`, so `duty <= self.max_duty_cycle()` (u16)
            #[allow(clippy::cast_possible_truncation)]
            {
                self.set_duty_cycle(duty as u16).await
            }
        }

        #[inline]
        async fn set_duty_cycle_percent(&mut self, percent: u8) -> Result<(), Self::Error> {
            self.set_duty_cycle_fraction(u16::from(percent), 100).await
        }
    }
}
