use super::{
    InputRegisters, Pins,
    aw9523b::{Register, read_register, write_register},
};
use crate::i2c::SharedI2cDevice;
use core::cell::RefCell;
use defmt::{debug, info};

pub struct PinControl<I2C: 'static> {
    bus: SharedI2cDevice<I2C>,
    pins: Option<Pins>,
}

impl<I2C, E> PinControl<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    pub fn new(bus: &'static RefCell<I2C>) -> Self {
        let bus = SharedI2cDevice::new(bus);
        let pins = Some(Pins::new());
        Self { bus, pins }
    }

    pub fn init(&mut self) -> Result<(), E> {
        for addr in [0x58, 0x59, 0x5A] {
            let id = read_register(&mut self.bus, addr, Register::ID)?;
            debug!("AW9523B with id {} found at address {}", id, addr);

            // Set port 0 to push pull mode and LED current to Imax
            write_register(&mut self.bus, addr, Register::CTL, 0b00010000)?;

            // Disable all interrupts
            write_register(&mut self.bus, addr, Register::INT_P0, 0b11111111)?;
            write_register(&mut self.bus, addr, Register::INT_P1, 0b11111111)?;
        }

        info!("IO expanders init");
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), E> {
        write_register(&mut self.bus, 0x58, Register::SW_RSTN, 0)?;
        write_register(&mut self.bus, 0x59, Register::SW_RSTN, 0)?;
        write_register(&mut self.bus, 0x5A, Register::SW_RSTN, 0)?;
        info!("IO expanders reset");
        Ok(())
    }

    pub fn pins(&mut self) -> Pins {
        self.pins.take().expect("can only take the pins once")
    }

    pub fn read_input_registers(&mut self) -> Result<InputRegisters, E> {
        InputRegisters::read(&mut self.bus)
    }
}
