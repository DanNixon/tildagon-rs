use embedded_hal::digital::{ErrorType, OutputPin};

#[derive(Default)]
pub(crate) struct FakePin {}

impl ErrorType for FakePin {
    type Error = embedded_hal::digital::ErrorKind;
}

impl OutputPin for FakePin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
