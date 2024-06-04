use core::{cell::RefCell, marker::ConstParamTy};
use embedded_hal::i2c::{ErrorType, I2c, Operation};

#[derive(ConstParamTy, PartialEq, Eq)]
#[repr(u8)]
pub enum BusNumber {
    Bus0 = 0b00000001,
    Bus1 = 0b00000010,
    Bus2 = 0b00000100,
    Bus3 = 0b00001000,
    Bus4 = 0b00010000,
    Bus5 = 0b00100000,
    Bus6 = 0b01000000,
    Bus7 = 0b10000000,
}

pub struct Bus<I2C: 'static, const BUS: BusNumber> {
    parent_bus: &'static RefCell<I2C>,
    mux_address: u8,
}

impl<I2C, const BUS: BusNumber> Bus<I2C, BUS> {
    pub fn new(bus: &'static RefCell<I2C>) -> Self {
        Self {
            parent_bus: bus,
            mux_address: 0x77,
        }
    }
}

impl<I2C, const BUS: BusNumber> ErrorType for Bus<I2C, BUS>
where
    I2C: I2c,
{
    type Error = I2C::Error;
}

impl<I2C, const BUS: BusNumber> I2c for Bus<I2C, BUS>
where
    I2C: I2c,
{
    #[inline]
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.parent_bus.borrow_mut();
        bus.write(self.mux_address, &[BUS as u8])?;
        bus.read(address, read)
    }

    #[inline]
    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let bus = &mut *self.parent_bus.borrow_mut();
        bus.write(self.mux_address, &[BUS as u8])?;
        bus.write(address, write)
    }

    #[inline]
    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let bus = &mut *self.parent_bus.borrow_mut();
        bus.write(self.mux_address, &[BUS as u8])?;
        bus.write_read(address, write, read)
    }

    #[inline]
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        let bus = &mut *self.parent_bus.borrow_mut();
        bus.write(self.mux_address, &[BUS as u8])?;
        bus.transaction(address, operations)
    }
}
