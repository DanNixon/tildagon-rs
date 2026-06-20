use crate::pins::UsbPins;
use defmt::Format;
use embedded_aw9523::{Output, async_traits::digital::OutputPin};
use embedded_hal::digital::PinState;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum UsbPort {
    In,
    Out,
}

pub struct UsbSwitch<I2C> {
    sw: Output<I2C>,
}

impl<I2C> UsbSwitch<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    pub fn new(r: UsbPins<I2C>) -> Self {
        Self { sw: r.usb_select }
    }

    pub async fn set(
        &mut self,
        port: UsbPort,
    ) -> Result<(), <Output<I2C> as embedded_hal::digital::ErrorType>::Error> {
        self.sw
            .set_state(match port {
                UsbPort::In => PinState::Low,
                UsbPort::Out => PinState::High,
            })
            .await
    }
}
