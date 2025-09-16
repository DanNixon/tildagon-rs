use super::pin::PinExt;
use core::marker::ConstParamTy;
use defmt::{Format, debug};
use embedded_hal::digital::PinState;
use embedded_hal_async::i2c::I2c;

#[derive(Format, Debug, Copy, Clone, PartialEq, Eq, ConstParamTy)]
pub enum Port {
    Port0,
    Port1,
}

#[derive(Format, Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum PinMode {
    Gpio,
    Led,
}

#[derive(Format, Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum GpioDirection {
    Input,
    Output,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub(super) enum Register {
    INPUT_P0 = 0x00,
    INPUT_P1 = 0x01,
    OUTPUT_P0 = 0x02,
    OUTPUT_P1 = 0x03,
    CONFIG_P0 = 0x04,
    CONFIG_P1 = 0x05,
    INT_P0 = 0x06,
    INT_P1 = 0x07,
    ID = 0x10,
    CTL = 0x11,
    LEDMS_P0 = 0x12,
    LEDMS_P1 = 0x13,
    DIM0_P10 = 0x20,
    DIM1_P11 = 0x21,
    DIM2_P12 = 0x22,
    DIM3_P13 = 0x23,
    DIM4_P00 = 0x24,
    DIM5_P01 = 0x25,
    DIM6_P02 = 0x26,
    DIM7_P03 = 0x27,
    DIM8_P04 = 0x28,
    DIM9_P05 = 0x29,
    DIM10_P06 = 0x2A,
    DIM11_P07 = 0x2B,
    DIM12_P14 = 0x2C,
    DIM13_P15 = 0x2D,
    DIM14_P16 = 0x2E,
    DIM15_P17 = 0x2F,
    SW_RSTN = 0x7F,
}

pub(super) async fn write_register<I2C, E>(
    bus: &mut I2C,
    addr: u8,
    register: Register,
    value: u8,
) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    bus.write(addr, &[register as u8, value]).await
}

pub(super) async fn read_register<I2C, E>(
    bus: &mut I2C,
    addr: u8,
    register: Register,
) -> Result<u8, E>
where
    I2C: I2c<Error = E>,
{
    let mut val = [0u8; 1];
    bus.write_read(addr, &[register as u8], &mut val)
        .await
        .and(Ok(val[0]))
}

pub(crate) async fn set_pin_mode<I2C, E, PIN: PinExt>(
    bus: &mut I2C,
    pin: &PIN,
    mode: PinMode,
) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    let register = match pin.port() {
        Port::Port0 => Register::LEDMS_P0,
        Port::Port1 => Register::LEDMS_P1,
    };

    let value = read_register(bus, pin.address(), register).await?;

    let value = match mode {
        PinMode::Gpio => value | pin.bit(),
        PinMode::Led => value & !pin.bit(),
    };

    write_register(bus, pin.address(), register, value).await?;

    debug!("Set mode of pin {} to {}", pin, mode);
    Ok(())
}

pub(crate) async fn set_io_direction<I2C, E, PIN: PinExt + ?Sized>(
    bus: &mut I2C,
    pin: &PIN,
    direction: GpioDirection,
) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    let register = match pin.port() {
        Port::Port0 => Register::CONFIG_P0,
        Port::Port1 => Register::CONFIG_P1,
    };

    let value = read_register(bus, pin.address(), register).await?;

    let value = match direction {
        GpioDirection::Input => value | pin.bit(),
        GpioDirection::Output => value & !pin.bit(),
    };

    write_register(bus, pin.address(), register, value).await?;

    debug!("Set IO direction of pin {} to {}", pin, direction);
    Ok(())
}

pub(crate) async fn set_io_state<I2C, E, PIN: PinExt + ?Sized>(
    bus: &mut I2C,
    pin: &PIN,
    state: PinState,
) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    let register = match pin.port() {
        Port::Port0 => Register::OUTPUT_P0,
        Port::Port1 => Register::OUTPUT_P1,
    };

    let value = read_register(bus, pin.address(), register).await?;

    let value = match state {
        PinState::Low => value & !pin.bit(),
        PinState::High => value | pin.bit(),
    };

    write_register(bus, pin.address(), register, value).await?;

    debug!("Set pin {} to {}", pin, state);
    Ok(())
}
