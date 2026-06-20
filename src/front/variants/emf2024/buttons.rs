use crate::{
    button_collection::{ButtonCollection, ButtonInformation},
    pins::ButtonPins,
};
use defmt::Format;
use strum::{EnumCount, EnumIter};

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy, EnumIter, EnumCount)]
pub enum SystemButton {
    A,
    B,
    C,
    D,
    E,
    F,
}

pub type SystemButtonCollection<I2C> = ButtonCollection<SystemButton, I2C, { SystemButton::COUNT }>;

impl<I2C> SystemButtonCollection<I2C> {
    pub fn new(p: ButtonPins<I2C>) -> Self {
        let state = [
            ButtonInformation {
                button: SystemButton::A,
                pin: p.btn1,
                state: None,
            },
            ButtonInformation {
                button: SystemButton::B,
                pin: p.btn2,
                state: None,
            },
            ButtonInformation {
                button: SystemButton::C,
                pin: p.btn3,
                state: None,
            },
            ButtonInformation {
                button: SystemButton::D,
                pin: p.btn4,
                state: None,
            },
            ButtonInformation {
                button: SystemButton::E,
                pin: p.btn5,
                state: None,
            },
            ButtonInformation {
                button: SystemButton::F,
                pin: p.btn6,
                state: None,
            },
        ];

        Self { state }
    }
}
