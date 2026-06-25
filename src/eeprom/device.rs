use defmt::Format;
use embassy_time::Duration;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum EepromError<I2cError> {
    Capacity,
    I2c(I2cError),
}

pub struct Eeprom<I2C> {
    bus: I2C,
    address: u8,
    size: u32,
}

impl<I2C> Eeprom<I2C> {
    pub fn new(bus: I2C, address: u8, size: u32) -> Self {
        Self { bus, address, size }
    }

    /// Transform this EEPROM driver into one with a different capacity.
    pub fn into_sized(self, size: u32) -> Self {
        Self {
            bus: self.bus,
            address: self.address,
            size,
        }
    }
}

impl<I2C> embedded_storage::Region for Eeprom<I2C> {
    fn contains(&self, address: u32) -> bool {
        address < self.size
    }
}

impl<I2C, E> embedded_storage::ReadStorage for Eeprom<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    type Error = EepromError<E>;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let len: u32 = bytes.len().try_into().map_err(|_| EepromError::Capacity)?;
        if offset.checked_add(len).ok_or(EepromError::Capacity)? > self.size {
            return Err(EepromError::Capacity);
        }

        let addr = offset.to_le_bytes();
        let addr = [addr[1], addr[0]];

        self.bus
            .write_read(self.address, &addr, bytes)
            .map_err(EepromError::I2c)
    }

    fn capacity(&self) -> usize {
        self.size as usize
    }
}

impl<I2C, E> embedded_storage::Storage for Eeprom<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let len: u32 = bytes.len().try_into().map_err(|_| EepromError::Capacity)?;
        if offset.checked_add(len).ok_or(EepromError::Capacity)? > self.size {
            return Err(EepromError::Capacity);
        }

        for (idx, &byte) in bytes.iter().enumerate() {
            let idx: u32 = idx.try_into().unwrap();
            let addr = offset + idx;

            let addr = addr.to_le_bytes();

            let buf = [addr[1], addr[0], byte];

            self.bus
                .write(self.address, &buf)
                .map_err(EepromError::I2c)?;

            embassy_time::block_for(Duration::from_millis(6));
        }

        Ok(())
    }
}
