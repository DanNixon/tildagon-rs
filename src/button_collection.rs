//! Driver for buttons connected via an AW9523.
//!
//! Intended for buttons/switches/joysticks on the front boards, but could also be of use for hexpansions.

use defmt::{Format, debug};
use embassy_time::{Duration, Instant};
use embedded_aw9523::{Input, InputRegisters, InputRegistersError};
use embedded_hal::digital::PinState;
use getset::Getters;
use heapless::Vec;

pub struct ButtonCollection<B, I2C, const N: usize> {
    pub(crate) state: [ButtonInformation<B, I2C>; N],
}

impl<B, I2C, E, const N: usize> ButtonCollection<B, I2C, N>
where
    B: core::fmt::Debug + Format + Copy,
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    pub fn update(
        &mut self,
        regs: &InputRegisters,
    ) -> Result<Vec<ButtonEvent<B>, N>, InputRegistersError> {
        let now = Instant::now();

        let mut events = Vec::new();

        for button in self.state.iter_mut() {
            let new = match regs.pin_state(&button.pin)? {
                PinState::Low => ButtonState::Pressed,
                PinState::High => ButtonState::Released,
            };

            if button.state.is_none() || button.state.unwrap().state != new {
                let new = TemporalButtonState {
                    time: now,
                    state: new,
                };

                events
                    .push(ButtonEvent {
                        button: button.button,
                        previous: button.state,
                        now: new,
                    })
                    .unwrap();

                button.state = Some(new);
            }
        }

        debug!("Button events: {}", events);
        Ok(events)
    }
}

pub(crate) struct ButtonInformation<B, I2C> {
    pub(crate) button: B,
    pub(crate) pin: Input<I2C>,
    pub(crate) state: Option<TemporalButtonState>,
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

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy, Getters)]
pub struct ButtonEvent<B> {
    #[getset(get = "pub")]
    button: B,

    #[getset(get = "pub")]
    previous: Option<TemporalButtonState>,

    #[getset(get = "pub")]
    now: TemporalButtonState,
}

impl<B> ButtonEvent<B> {
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
