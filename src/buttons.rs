use crate::pins::{
    ButtonPins, InputRegisters,
    aw9523b::{GpioDirection, PinMode, set_io_direction, set_pin_mode},
};
use defmt::{Format, debug};
use embassy_time::{Duration, Instant};
use getset::Getters;
use heapless::Vec;

const BUTTON_COUNT: usize = 6;

pub struct Buttons {
    pins: ButtonPins,
    state: [Option<TemporalButtonState>; BUTTON_COUNT],
}

impl Buttons {
    pub async fn try_new<I2C, E>(mut i2c: I2C, pins: ButtonPins) -> Result<Self, E>
    where
        I2C: embedded_hal_async::i2c::I2c<Error = E>,
    {
        macro_rules! setup_button {
            ($bus:expr, $pin:expr) => {
                set_pin_mode($bus, &$pin, PinMode::Gpio).await?;
                set_io_direction($bus, &$pin, GpioDirection::Input).await?;
            };
        }

        setup_button!(&mut i2c, pins.btn1);
        setup_button!(&mut i2c, pins.btn2);
        setup_button!(&mut i2c, pins.btn3);
        setup_button!(&mut i2c, pins.btn4);
        setup_button!(&mut i2c, pins.btn5);
        setup_button!(&mut i2c, pins.btn6);

        let state = [None; BUTTON_COUNT];

        Ok(Self { pins, state })
    }

    pub fn update(&mut self, regs: &InputRegisters) -> Vec<ButtonEvent, BUTTON_COUNT> {
        let now = Instant::now();
        let states_now = states_from_registers(&self.pins, regs);

        let mut events = Vec::new();

        for (idx, old) in self.state.iter_mut().enumerate() {
            let new = &states_now[idx];

            if old.is_none() || old.unwrap().state != *new {
                let new = TemporalButtonState {
                    time: now,
                    state: *new,
                };

                events
                    .push(ButtonEvent {
                        button: Button::from_index(idx),
                        previous: *old,
                        now: new,
                    })
                    .unwrap();

                *old = Some(new);
            }
        }

        debug!("Button events: {}", events);
        events
    }
}

#[cfg(feature = "top-board-2024")]
#[derive(Debug, Format, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    A,
    B,
    C,
    D,
    E,
    F,
}

#[cfg(feature = "top-board-2024")]
impl Button {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy, Getters)]
pub struct ButtonEvent {
    #[getset(get = "pub")]
    button: Button,

    #[getset(get = "pub")]
    previous: Option<TemporalButtonState>,

    #[getset(get = "pub")]
    now: TemporalButtonState,
}

impl ButtonEvent {
    pub fn pressed(&self) -> bool {
        self.now.state == ButtonState::Pressed
    }

    pub fn released(&self) -> bool {
        self.now.state == ButtonState::Released
    }

    pub fn duration(&self) -> Option<Duration> {
        self.previous.map(|previous| self.now.time - previous.time)
    }
}

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy, Getters)]
pub struct TemporalButtonState {
    #[getset(get = "pub")]
    time: Instant,

    #[getset(get = "pub")]
    state: ButtonState,
}

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy)]
pub enum ButtonState {
    Pressed,
    Released,
}

fn states_from_registers(pins: &ButtonPins, regs: &InputRegisters) -> [ButtonState; BUTTON_COUNT] {
    [
        regs.pin_state(&pins.btn1),
        regs.pin_state(&pins.btn2),
        regs.pin_state(&pins.btn3),
        regs.pin_state(&pins.btn4),
        regs.pin_state(&pins.btn5),
        regs.pin_state(&pins.btn6),
    ]
    .map(|high| {
        if high {
            ButtonState::Released
        } else {
            ButtonState::Pressed
        }
    })
}
