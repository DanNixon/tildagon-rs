use crate::pins::ButtonPins;
use defmt::{Format, debug};
use embassy_time::{Duration, Instant};
use embedded_aw9523::InputRegisters;
use embedded_hal::digital::PinState;
use getset::Getters;
use heapless::Vec;

const BUTTON_COUNT: usize = 6;

pub struct Buttons<I2C> {
    pins: ButtonPins<I2C>,
    state: [Option<TemporalButtonState>; BUTTON_COUNT],
}

impl<I2C, E> Buttons<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    pub fn new(pins: ButtonPins<I2C>) -> Self {
        let state = [None; BUTTON_COUNT];
        Self { pins, state }
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

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    A,
    B,
    C,
    D,
    E,
    F,
}

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

fn states_from_registers<I2C>(
    pins: &ButtonPins<I2C>,
    regs: &InputRegisters,
) -> [ButtonState; BUTTON_COUNT] {
    [
        regs.pin_state(&pins.btn1),
        regs.pin_state(&pins.btn2),
        regs.pin_state(&pins.btn3),
        regs.pin_state(&pins.btn4),
        regs.pin_state(&pins.btn5),
        regs.pin_state(&pins.btn6),
    ]
    .map(|state| match state.unwrap() {
        PinState::Low => ButtonState::Pressed,
        PinState::High => ButtonState::Released,
    })
}
