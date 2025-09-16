use super::{
    aw9523b::{Port, Register},
    pin::PinExt,
};
use defmt::Format;

#[derive(Debug, Format, PartialEq, Eq, Clone)]
pub struct InputRegisters {
    a0x58_port0: u8,
    a0x58_port1: u8,
    a0x59_port0: u8,
    a0x59_port1: u8,
    a0x5a_port0: u8,
    a0x5a_port1: u8,
}

impl InputRegisters {
    pub(crate) async fn read<I2C, E>(i2c: &mut I2C) -> Result<Self, E>
    where
        I2C: embedded_hal_async::i2c::I2c<Error = E>,
    {
        let mut val = [0u8; 2];

        i2c.write_read(
            0x58,
            &[Register::INPUT_P0 as u8, Register::INPUT_P1 as u8],
            &mut val,
        )
        .await?;
        let (a0x58_port0, a0x58_port1) = (val[0], val[1]);

        i2c.write_read(
            0x59,
            &[Register::INPUT_P0 as u8, Register::INPUT_P1 as u8],
            &mut val,
        )
        .await?;
        let (a0x59_port0, a0x59_port1) = (val[0], val[1]);

        i2c.write_read(
            0x5a,
            &[Register::INPUT_P0 as u8, Register::INPUT_P1 as u8],
            &mut val,
        )
        .await?;
        let (a0x5a_port0, a0x5a_port1) = (val[0], val[1]);

        Ok(Self {
            a0x58_port0,
            a0x58_port1,
            a0x59_port0,
            a0x59_port1,
            a0x5a_port0,
            a0x5a_port1,
        })
    }

    pub(crate) fn pin_state<PIN: PinExt>(&self, pin: &PIN) -> bool {
        let (reg, bit) = match pin.address() {
            0x58 => match pin.port() {
                Port::Port0 => (self.a0x58_port0, pin.bit()),
                Port::Port1 => (self.a0x58_port1, pin.bit()),
            },
            0x59 => match pin.port() {
                Port::Port0 => (self.a0x59_port0, pin.bit()),
                Port::Port1 => (self.a0x59_port1, pin.bit()),
            },
            0x5A => match pin.port() {
                Port::Port0 => (self.a0x5a_port0, pin.bit()),
                Port::Port1 => (self.a0x5a_port1, pin.bit()),
            },
            _ => panic!("Invalid address"),
        };
        reg & bit != 0
    }
}
