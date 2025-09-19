use core::marker::ConstParamTy;
use defmt::{Format, debug};
use embedded_hal_async::i2c::{ErrorType, I2c, Operation};

use super::SharedI2cBus;

#[derive(Format, ConstParamTy, PartialEq, Eq)]
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

pub struct Bus<BUS: 'static, const N: BusNumber> {
    parent_bus: &'static SharedI2cBus<BUS>,
    mux_address: u8,
}

impl<BUS, const N: BusNumber> Bus<BUS, N> {
    pub fn new(bus: &'static SharedI2cBus<BUS>) -> Self {
        Self {
            parent_bus: bus,
            mux_address: 0x77,
        }
    }
}

impl<BUS, const N: BusNumber> ErrorType for Bus<BUS, N>
where
    BUS: ErrorType,
{
    type Error = BUS::Error;
}

impl<BUS, const N: BusNumber> I2c for Bus<BUS, N>
where
    BUS: I2c,
{
    #[inline]
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let mut bus = self.parent_bus.lock().await;
        bus.write(self.mux_address, &[N as u8]).await?;
        debug!("Selected {}", N);
        bus.read(address, read).await
    }

    #[inline]
    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let mut bus = self.parent_bus.lock().await;
        bus.write(self.mux_address, &[N as u8]).await?;
        debug!("Selected {}", N);
        bus.write(address, write).await
    }

    #[inline]
    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut bus = self.parent_bus.lock().await;
        bus.write(self.mux_address, &[N as u8]).await?;
        debug!("Selected {}", N);
        bus.write_read(address, write, read).await
    }

    #[inline]
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        let mut bus = self.parent_bus.lock().await;
        bus.write(self.mux_address, &[N as u8]).await?;
        debug!("Selected {}", N);
        bus.transaction(address, operations).await
    }
}
