use crate::pins::HexpansionDetectPins;
use defmt::{Format, debug};
use embassy_time::Instant;
use embedded_aw9523::{
    Input, InputRegisters, Output, PinConfiguration, async_traits::digital::OutputPin,
};
use embedded_hal::digital::PinState;
use getset::Getters;
use heapless::{Vec, index_map::FnvIndexMap};
use strum::{EnumCount, EnumIter, IntoEnumIterator};

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy, Hash, EnumIter, EnumCount)]
pub enum HexpansionSlot {
    A,
    B,
    C,
    D,
    E,
    F,
}

pub struct HexpansionSlotControl<I2C> {
    state: FnvIndexMap<HexpansionSlot, SlotState<I2C>, 8>,
}

impl<I2C, E> HexpansionSlotControl<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    pub async fn new(pins: HexpansionDetectPins<I2C>) -> Result<Self, E> {
        let mut state = FnvIndexMap::new();

        let _ = state.insert(
            HexpansionSlot::A,
            SlotState {
                mode: SlotModeState::new(pins.a).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionSlot::B,
            SlotState {
                mode: SlotModeState::new(pins.b).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionSlot::C,
            SlotState {
                mode: SlotModeState::new(pins.c).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionSlot::D,
            SlotState {
                mode: SlotModeState::new(pins.d).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionSlot::E,
            SlotState {
                mode: SlotModeState::new(pins.e).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionSlot::F,
            SlotState {
                mode: SlotModeState::new(pins.f).await?,
                notified: false,
            },
        );

        Ok(Self { state })
    }

    pub async fn set_enabled(&mut self, slot: HexpansionSlot, enabled: bool) -> Result<(), E> {
        let mut state = self.state.remove(&slot).unwrap();
        state.mode = if enabled {
            state.mode.enable().await?
        } else {
            state.mode.disable().await?
        };
        state.notified = false;
        let _ = self.state.insert(slot, state);
        Ok(())
    }

    pub fn update(
        &mut self,
        regs: &InputRegisters,
    ) -> Vec<HexpansionSlotEvent, { HexpansionSlot::COUNT }> {
        let now = Instant::now();

        let mut events = Vec::new();

        for hex_slot in HexpansionSlot::iter() {
            let slot_state = self.state.get_mut(&hex_slot).unwrap();

            if let SlotModeState::Enabled {
                ref pin,
                ref mut present,
            } = slot_state.mode
            {
                match regs.pin_state(pin).unwrap() {
                    PinState::Low => {
                        if !*present {
                            slot_state.notified = false;
                        }
                        *present = true;
                    }
                    PinState::High => {
                        if *present {
                            slot_state.notified = false;
                        }
                        *present = false;
                    }
                }
            }

            if !slot_state.notified {
                let _ = events.push(HexpansionSlotEvent {
                    slot: hex_slot,
                    time: now,
                    state: slot_state.state(),
                });
                slot_state.notified = true;
            }
        }

        debug!("Hexpansion events: {}", events);
        events
    }
}

struct SlotState<I2C> {
    mode: SlotModeState<I2C>,
    notified: bool,
}

impl<I2C> SlotState<I2C> {
    fn state(&self) -> HexpansionState {
        match &self.mode {
            SlotModeState::Disabled { pin: _ } => HexpansionState::Disabled,
            SlotModeState::Enabled { pin: _, present } => {
                if *present {
                    HexpansionState::Occupied
                } else {
                    HexpansionState::Empty
                }
            }
        }
    }
}

enum SlotModeState<I2C> {
    Disabled { pin: Output<I2C> },
    Enabled { pin: Input<I2C>, present: bool },
}

impl<I2C, E> SlotModeState<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    async fn new<PIN: PinConfiguration<I2C, E>>(pin: PIN) -> Result<Self, E> {
        let mut pin = pin.try_into_output().await?;
        pin.set_high().await.unwrap(); // TODO
        Ok(Self::Disabled { pin })
    }

    async fn disable(self) -> Result<Self, E> {
        let mut pin = match self {
            SlotModeState::Disabled { pin } => pin.try_into_output().await?,
            SlotModeState::Enabled { pin, present: _ } => pin.try_into_output().await?,
        };
        pin.set_high().await.unwrap(); // TODO
        Ok(Self::Disabled { pin })
    }

    async fn enable(self) -> Result<Self, E> {
        let pin = match self {
            SlotModeState::Disabled { pin } => pin.try_into_input().await?,
            SlotModeState::Enabled { pin, present: _ } => pin.try_into_input().await?,
        };
        Ok(Self::Enabled {
            pin,
            present: false,
        })
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
