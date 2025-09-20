use super::types::Error;
use embedded_hal_async::i2c::{I2c, SevenBitAddress};

/// Default I2C address of BMI270
const BMI270_I2C_ADDR: u8 = 0x68;
/// Alternative I2C address when SDO is pulled high
const BMI270_I2C_ADDR_ALT: u8 = 0x69;

pub struct I2cInterface<I2C> {
    pub i2c: I2C,
    pub address: u8,
}

/// I2c address.
#[derive(Debug, Default, Clone, Copy)]
pub enum I2cAddr {
    /// Use the default i2c address, 0x68.
    #[default]
    Default,
    /// Use alternative 0x69 as the i2c address (selected when SDO is pulled high).
    Alternative,
}

impl I2cAddr {
    pub fn addr(self) -> SevenBitAddress {
        match self {
            I2cAddr::Default => BMI270_I2C_ADDR,
            I2cAddr::Alternative => BMI270_I2C_ADDR_ALT,
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait WriteData {
    type Error;
    async fn write(&mut self, payload: &mut [u8]) -> Result<(), Self::Error>;
    async fn write_reg(&mut self, register: u8, data: u8) -> Result<(), Self::Error>;
}

#[allow(async_fn_in_trait)]
pub trait ReadData {
    type Error;
    async fn read(&mut self, payload: &mut [u8]) -> Result<(), Self::Error>;
    async fn read_reg(&mut self, register: u8) -> Result<u8, Self::Error>;
}

impl<I2C, E> WriteData for I2cInterface<I2C>
where
    I2C: I2c<Error = E>,
{
    type Error = Error<I2C::Error>;

    async fn write(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c
            .write(self.address, payload)
            .await
            .map_err(Error::Comm)
    }

    async fn write_reg(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [register, data];
        self.i2c
            .write(self.address, &payload)
            .await
            .map_err(Error::Comm)
    }
}

impl<I2C, E> ReadData for I2cInterface<I2C>
where
    I2C: I2c<Error = E>,
{
    type Error = Error<I2C::Error>;

    async fn read(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c
            .write_read(self.address, &[payload[0]], &mut payload[1..])
            .await
            .map_err(Error::Comm)
    }

    async fn read_reg(&mut self, register: u8) -> Result<u8, Self::Error> {
        let mut data = [0];
        self.i2c
            .write_read(self.address, &[register], &mut data)
            .await
            .map_err(Error::Comm)
            .and(Ok(data[0]))
    }
}
