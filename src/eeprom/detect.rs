use crate::{eeprom::Eeprom, i2c::BlockingI2cDeviceWrapper};

/// A Zetta ZD24C64A with A_{0,1,2} pulled high
pub const ZD24C64A_HHH_ADDR: u8 = 0b1010111;

// TODO: document what part/config this is for
pub const OTHER_ADDR: u8 = 0x50;

pub async fn detect_eeprom_addr<I2C>(bus: &mut I2C) -> Result<u8, ()>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    for address in [ZD24C64A_HHH_ADDR, OTHER_ADDR] {
        if bus.read(address, &mut [0]).await.is_ok() {
            return Ok(address);
        }
    }

    Err(())
}

pub async fn detect_eeprom<I2C>(mut bus: I2C) -> Result<Eeprom<BlockingI2cDeviceWrapper<I2C>>, ()>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let addr = detect_eeprom_addr(&mut bus).await?;

    // Just enough to read the hexpansion header, then the device driver can be resized
    const INITIAL_SIZE: u32 = 32;

    Ok(Eeprom::new(
        BlockingI2cDeviceWrapper::new(bus),
        addr,
        INITIAL_SIZE,
    ))
}
