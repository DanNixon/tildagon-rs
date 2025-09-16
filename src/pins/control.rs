use super::{
    InputRegisters, Pins,
    aw9523b::{Register, read_register, write_register},
};
use crate::i2c::{SharedI2cBus, SharedI2cDevice};
use defmt::{debug, info};
use embassy_embedded_hal::shared_bus::I2cDeviceError;

pub struct PinControl<I2C: 'static> {
    bus: SharedI2cDevice<I2C>,
    pins: Option<Pins>,
}

impl<I2C, E> PinControl<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    pub fn new(bus: &'static SharedI2cBus<I2C>) -> Self {
        let bus = SharedI2cDevice::new(bus);
        let pins = Some(Pins::new());
        Self { bus, pins }
    }

    pub async fn init(&mut self) -> Result<(), I2cDeviceError<E>> {
        for addr in [0x58, 0x59, 0x5A] {
            let id = read_register(&mut self.bus, addr, Register::ID).await?;
            debug!("AW9523B with id {} found at address {}", id, addr);

            // Set port 0 to push pull mode and LED current to Imax
            write_register(&mut self.bus, addr, Register::CTL, 0b00010000).await?;

            // Disable all interrupts
            write_register(&mut self.bus, addr, Register::INT_P0, 0b11111111).await?;
            write_register(&mut self.bus, addr, Register::INT_P1, 0b11111111).await?;
        }

        info!("IO expanders init");
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), I2cDeviceError<E>> {
        write_register(&mut self.bus, 0x58, Register::SW_RSTN, 0).await?;
        write_register(&mut self.bus, 0x59, Register::SW_RSTN, 0).await?;
        write_register(&mut self.bus, 0x5A, Register::SW_RSTN, 0).await?;
        info!("IO expanders reset");
        Ok(())
    }

    pub fn pins(&mut self) -> Pins {
        self.pins.take().expect("can only take the pins once")
    }

    pub async fn read_input_registers(&mut self) -> Result<InputRegisters, I2cDeviceError<E>> {
        InputRegisters::read(&mut self.bus).await
    }
}
