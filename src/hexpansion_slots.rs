use crate::pins::{
    HexpansionDetectPins, InputRegisters,
    aw9523b::{GpioDirection, PinMode, set_io_direction, set_io_state, set_pin_mode},
    pin::PinExt,
};
use defmt::{Format, debug};
use embassy_time::Instant;
use embedded_hal::{digital::PinState, i2c::I2c};
use getset::Getters;
use heapless::Vec;

const HEXPANSION_SLOT_COUNT: usize = 6;

pub struct HexpansionSlotControl<I2C> {
    i2c: I2C,
    pins: HexpansionDetectPins,
    state: [InnerState; HEXPANSION_SLOT_COUNT],
}

impl<I2C, E> HexpansionSlotControl<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn try_new(mut i2c: I2C, pins: HexpansionDetectPins) -> Result<Self, E> {
        macro_rules! setup_pin {
            ($bus:expr, $pin:expr) => {
                set_pin_mode($bus, &$pin, PinMode::Gpio)?;
                set_io_direction($bus, &$pin, GpioDirection::Output)?;
                set_io_state($bus, &$pin, PinState::High)?;
            };
        }

        setup_pin!(&mut i2c, pins.a);
        setup_pin!(&mut i2c, pins.b);
        setup_pin!(&mut i2c, pins.c);
        setup_pin!(&mut i2c, pins.d);
        setup_pin!(&mut i2c, pins.e);
        setup_pin!(&mut i2c, pins.f);

        let state = [InnerState {
            state: Some(HexpansionState::Disabled),
            notified: false,
        }; 6];

        Ok(Self { i2c, pins, state })
    }

    pub fn set_enabled(&mut self, slot: HexpansionSlot, enabled: bool) -> Result<(), E> {
        let pin: &dyn PinExt = match slot {
            HexpansionSlot::A => &self.pins.a,
            HexpansionSlot::B => &self.pins.b,
            HexpansionSlot::C => &self.pins.c,
            HexpansionSlot::D => &self.pins.d,
            HexpansionSlot::E => &self.pins.e,
            HexpansionSlot::F => &self.pins.f,
        };

        let state = if enabled {
            set_io_direction(&mut self.i2c, pin, GpioDirection::Input)?;
            None
        } else {
            set_io_direction(&mut self.i2c, pin, GpioDirection::Output)?;
            set_io_state(&mut self.i2c, pin, PinState::High)?;
            Some(HexpansionState::Disabled)
        };

        self.state[slot as usize] = InnerState {
            state,
            notified: false,
        };

        Ok(())
    }

    pub fn update(
        &mut self,
        regs: &InputRegisters,
    ) -> Vec<HexpansionSlotEvent, HEXPANSION_SLOT_COUNT> {
        let now = Instant::now();
        let states_now = states_from_registers(&self.pins, regs);

        let mut events = Vec::new();

        for (idx, old) in self.state.iter_mut().enumerate() {
            let new = &states_now[idx];

            if old.state == Some(HexpansionState::Disabled) {
                if old.notified {
                    continue;
                } else {
                    events
                        .push(HexpansionSlotEvent {
                            slot: HexpansionSlot::from_index(idx),
                            time: now,
                            state: HexpansionState::Disabled,
                        })
                        .unwrap();

                    old.notified = true;
                }
            } else if old.state != Some(*new) {
                events
                    .push(HexpansionSlotEvent {
                        slot: HexpansionSlot::from_index(idx),
                        time: now,
                        state: *new,
                    })
                    .unwrap();

                old.state = Some(*new);
                old.notified = true;
            }
        }

        debug!("Hexpansion events: {}", events);
        events
    }
}

#[derive(Clone, Copy)]
struct InnerState {
    state: Option<HexpansionState>,
    notified: bool,
}

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy)]
#[repr(usize)]
pub enum HexpansionSlot {
    A,
    B,
    C,
    D,
    E,
    F,
}

impl HexpansionSlot {
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
pub struct HexpansionSlotEvent {
    /// The slot this event is about
    #[getset(get = "pub")]
    slot: HexpansionSlot,

    /// The time the event happened
    #[getset(get = "pub")]
    time: Instant,

    /// The state the slot is now in
    #[getset(get = "pub")]
    state: HexpansionState,
}

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy)]
pub enum HexpansionState {
    /// The slot is disabled and will not supply power
    Disabled,

    /// The slot is enabled, but no hexpansion is demanding power from it
    Empty,

    /// The slot is enabled and contains a hexpansion that is demanding power
    Occupied,
}

fn states_from_registers(
    pins: &HexpansionDetectPins,
    regs: &InputRegisters,
) -> [HexpansionState; HEXPANSION_SLOT_COUNT] {
    [
        regs.pin_state(&pins.a),
        regs.pin_state(&pins.b),
        regs.pin_state(&pins.c),
        regs.pin_state(&pins.d),
        regs.pin_state(&pins.e),
        regs.pin_state(&pins.f),
    ]
    .map(|high| {
        if high {
            HexpansionState::Empty
        } else {
            HexpansionState::Occupied
        }
    })
}
