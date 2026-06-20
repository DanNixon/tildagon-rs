use crate::i2c::{SharedI2cDevice, SharingRawMutex, SystemI2cBus};
use embassy_sync::mutex::Mutex;
use embedded_aw9523::{Address, Aw9523, Input, InputRegisters, Output, PinConfiguration};

pub struct PinControl {
    system_bus: SharedI2cDevice<SystemI2cBus>,
    pins: Option<Pins<SharedI2cDevice<SystemI2cBus>>>,
}

impl PinControl {
    pub async fn new(
        system_bus: &'static Mutex<SharingRawMutex, SystemI2cBus>,
    ) -> Result<Self, <SharedI2cDevice<SystemI2cBus> as embedded_hal_async::i2c::ErrorType>::Error>
    {
        let mut addr58 = Aw9523::new(SharedI2cDevice::new(system_bus), Address::Addr58).await?;
        let mut addr59 = Aw9523::new(SharedI2cDevice::new(system_bus), Address::Addr59).await?;
        let mut addr5a = Aw9523::new(SharedI2cDevice::new(system_bus), Address::Addr5A).await?;

        addr58.init().await?;
        addr59.init().await?;
        addr5a.init().await?;

        let pins58 = addr58.pins();
        let pins59 = addr59.pins();
        let pins5a = addr5a.pins();

        let pins = Pins::new(pins58, pins59, pins5a).await?;

        Ok(Self {
            system_bus: SharedI2cDevice::new(system_bus),
            pins: Some(pins),
        })
    }

    pub fn pins(&mut self) -> Pins<SharedI2cDevice<SystemI2cBus>> {
        self.pins.take().expect("can only take the pins once")
    }

    pub async fn read_system_bus_input_registers(
        &mut self,
    ) -> Result<
        InputRegisters,
        <SharedI2cDevice<SystemI2cBus> as embedded_hal_async::i2c::ErrorType>::Error,
    > {
        InputRegisters::read(
            &mut self.system_bus,
            &[Address::Addr58, Address::Addr59, Address::Addr5A],
        )
        .await
    }
}

pub struct Pins<SysI2C> {
    pub other: OtherPins<SysI2C>,
    pub led: LedPins<SysI2C>,
    pub top_board: TopBoardPins<SysI2C>,
    pub hexpansion_detect: HexpansionDetectPins<SysI2C>,
    pub buttons: ButtonPins<SysI2C>,
    pub hexpansion_a: HexpansionAPins<SysI2C>,
    pub hexpansion_b: HexpansionBPins<SysI2C>,
    pub hexpansion_c: HexpansionCPins<SysI2C>,
    pub hexpansion_d: HexpansionDPins<SysI2C>,
    pub hexpansion_e: HexpansionEPins<SysI2C>,
    pub hexpansion_f: HexpansionFPins<SysI2C>,
}

impl<SysI2C, E> Pins<SysI2C>
where
    SysI2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    async fn new(
        addr58_pins: embedded_aw9523::Pins<SysI2C>,
        addr59_pins: embedded_aw9523::Pins<SysI2C>,
        addr5a_pins: embedded_aw9523::Pins<SysI2C>,
    ) -> Result<Self, E> {
        Ok(Self {
            other: OtherPins {
                vbus_sw: addr5a_pins.port0_pin4.try_into_output().await?,
                usb_select: addr5a_pins.port0_pin5.try_into_output().await?,
                accel_int: addr58_pins.port0_pin1,
            },
            led: LedPins {
                power_enable: addr5a_pins.port0_pin2.try_into_output().await?,
            },
            top_board: TopBoardPins {
                ls_1: addr5a_pins.port1_pin7,
                ls_2: addr5a_pins.port1_pin6,
            },
            hexpansion_detect: HexpansionDetectPins {
                a: addr5a_pins.port1_pin4.try_into_output().await?,
                b: addr5a_pins.port1_pin5.try_into_output().await?,
                c: addr59_pins.port1_pin0.try_into_output().await?,
                d: addr59_pins.port1_pin1.try_into_output().await?,
                e: addr59_pins.port1_pin2.try_into_output().await?,
                f: addr59_pins.port1_pin3.try_into_output().await?,
            },
            buttons: ButtonPins {
                btn1: addr5a_pins.port0_pin6,
                btn2: addr5a_pins.port0_pin7,
                btn3: addr59_pins.port0_pin0,
                btn4: addr59_pins.port0_pin1,
                btn5: addr59_pins.port0_pin2,
                btn6: addr59_pins.port0_pin3,
            },
            hexpansion_a: HexpansionAPins {
                ls_1: addr5a_pins.port0_pin3,
                ls_2: addr5a_pins.port1_pin0,
                ls_3: addr5a_pins.port1_pin1,
                ls_4: addr5a_pins.port1_pin2,
                ls_5: addr5a_pins.port1_pin3,
            },
            hexpansion_b: HexpansionBPins {
                ls_1: addr5a_pins.port0_pin0,
                ls_2: addr5a_pins.port0_pin1,
                ls_3: addr59_pins.port1_pin5,
                ls_4: addr59_pins.port1_pin6,
                ls_5: addr59_pins.port1_pin7,
            },
            hexpansion_c: HexpansionCPins {
                ls_1: addr59_pins.port0_pin4,
                ls_2: addr59_pins.port0_pin5,
                ls_3: addr59_pins.port0_pin6,
                ls_4: addr59_pins.port0_pin7,
                ls_5: addr59_pins.port1_pin4,
            },
            hexpansion_d: HexpansionDPins {
                ls_1: addr58_pins.port1_pin0,
                ls_2: addr58_pins.port1_pin1,
                ls_3: addr58_pins.port1_pin2,
                ls_4: addr58_pins.port1_pin3,
                ls_5: addr58_pins.port0_pin0,
            },
            hexpansion_e: HexpansionEPins {
                ls_1: addr58_pins.port0_pin2,
                ls_2: addr58_pins.port0_pin3,
                ls_3: addr58_pins.port0_pin4,
                ls_4: addr58_pins.port0_pin5,
                ls_5: addr58_pins.port0_pin6,
            },
            hexpansion_f: HexpansionFPins {
                ls_1: addr58_pins.port0_pin7,
                ls_2: addr58_pins.port1_pin4,
                ls_3: addr58_pins.port1_pin5,
                ls_4: addr58_pins.port1_pin6,
                ls_5: addr58_pins.port1_pin7,
            },
        })
    }
}

pub struct OtherPins<SysI2C> {
    pub vbus_sw: Output<SysI2C>,
    pub usb_select: Output<SysI2C>,
    pub accel_int: Input<SysI2C>,
}

pub struct LedPins<SysI2C> {
    pub power_enable: Output<SysI2C>,
}

pub struct TopBoardPins<SysI2C> {
    pub ls_1: Input<SysI2C>,
    pub ls_2: Input<SysI2C>,
}

pub struct HexpansionDetectPins<SysI2C> {
    pub a: Output<SysI2C>,
    pub b: Output<SysI2C>,
    pub c: Output<SysI2C>,
    pub d: Output<SysI2C>,
    pub e: Output<SysI2C>,
    pub f: Output<SysI2C>,
}

pub struct ButtonPins<SysI2C> {
    pub btn1: Input<SysI2C>,
    pub btn2: Input<SysI2C>,
    pub btn3: Input<SysI2C>,
    pub btn4: Input<SysI2C>,
    pub btn5: Input<SysI2C>,
    pub btn6: Input<SysI2C>,
}

pub struct HexpansionAPins<SysI2C> {
    pub ls_1: Input<SysI2C>,
    pub ls_2: Input<SysI2C>,
    pub ls_3: Input<SysI2C>,
    pub ls_4: Input<SysI2C>,
    pub ls_5: Input<SysI2C>,
}

pub struct HexpansionBPins<SysI2C> {
    pub ls_1: Input<SysI2C>,
    pub ls_2: Input<SysI2C>,
    pub ls_3: Input<SysI2C>,
    pub ls_4: Input<SysI2C>,
    pub ls_5: Input<SysI2C>,
}

pub struct HexpansionCPins<SysI2C> {
    pub ls_1: Input<SysI2C>,
    pub ls_2: Input<SysI2C>,
    pub ls_3: Input<SysI2C>,
    pub ls_4: Input<SysI2C>,
    pub ls_5: Input<SysI2C>,
}

pub struct HexpansionDPins<SysI2C> {
    pub ls_1: Input<SysI2C>,
    pub ls_2: Input<SysI2C>,
    pub ls_3: Input<SysI2C>,
    pub ls_4: Input<SysI2C>,
    pub ls_5: Input<SysI2C>,
}

pub struct HexpansionEPins<SysI2C> {
    pub ls_1: Input<SysI2C>,
    pub ls_2: Input<SysI2C>,
    pub ls_3: Input<SysI2C>,
    pub ls_4: Input<SysI2C>,
    pub ls_5: Input<SysI2C>,
}

pub struct HexpansionFPins<SysI2C> {
    pub ls_1: Input<SysI2C>,
    pub ls_2: Input<SysI2C>,
    pub ls_3: Input<SysI2C>,
    pub ls_4: Input<SysI2C>,
    pub ls_5: Input<SysI2C>,
}
