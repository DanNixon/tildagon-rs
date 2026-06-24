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
pub enum HexpansionPort {
    A,
    B,
    C,
    D,
    E,
    F,
}

pub struct HexpansionPortControl<I2C> {
    state: FnvIndexMap<HexpansionPort, PortState<I2C>, 8>,
}

impl<I2C, E> HexpansionPortControl<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    pub async fn new(pins: HexpansionDetectPins<I2C>) -> Result<Self, E> {
        let mut state = FnvIndexMap::new();

        let _ = state.insert(
            HexpansionPort::A,
            PortState {
                mode: PortModeState::new(pins.a).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionPort::B,
            PortState {
                mode: PortModeState::new(pins.b).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionPort::C,
            PortState {
                mode: PortModeState::new(pins.c).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionPort::D,
            PortState {
                mode: PortModeState::new(pins.d).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionPort::E,
            PortState {
                mode: PortModeState::new(pins.e).await?,
                notified: false,
            },
        );
        let _ = state.insert(
            HexpansionPort::F,
            PortState {
                mode: PortModeState::new(pins.f).await?,
                notified: false,
            },
        );

        Ok(Self { state })
    }

    pub async fn set_enabled(&mut self, port: HexpansionPort, enabled: bool) -> Result<(), E> {
        let mut state = self.state.remove(&port).unwrap();
        state.mode = if enabled {
            state.mode.enable().await?
        } else {
            state.mode.disable().await?
        };
        state.notified = false;
        let _ = self.state.insert(port, state);
        Ok(())
    }

    pub fn update(
        &mut self,
        regs: &InputRegisters,
    ) -> Vec<HexpansionPortEvent, { HexpansionPort::COUNT }> {
        let now = Instant::now();

        let mut events = Vec::new();

        for hex_port in HexpansionPort::iter() {
            let port_state = self.state.get_mut(&hex_port).unwrap();

            if let PortModeState::Enabled {
                ref pin,
                ref mut present,
            } = port_state.mode
            {
                match regs.pin_state(pin).unwrap() {
                    PinState::Low => {
                        if !*present {
                            port_state.notified = false;
                        }
                        *present = true;
                    }
                    PinState::High => {
                        if *present {
                            port_state.notified = false;
                        }
                        *present = false;
                    }
                }
            }

            if !port_state.notified {
                let _ = events.push(HexpansionPortEvent {
                    port: hex_port,
                    time: now,
                    state: port_state.state(),
                });
                port_state.notified = true;
            }
        }

        debug!("Hexpansion events: {}", events);
        events
    }
}

struct PortState<I2C> {
    mode: PortModeState<I2C>,
    notified: bool,
}

impl<I2C> PortState<I2C> {
    fn state(&self) -> HexpansionState {
        match &self.mode {
            PortModeState::Disabled { pin: _ } => HexpansionState::Disabled,
            PortModeState::Enabled { pin: _, present } => {
                if *present {
                    HexpansionState::Occupied
                } else {
                    HexpansionState::Empty
                }
            }
        }
    }
}

enum PortModeState<I2C> {
    Disabled { pin: Output<I2C> },
    Enabled { pin: Input<I2C>, present: bool },
}

impl<I2C, E> PortModeState<I2C>
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
            PortModeState::Disabled { pin } => pin.try_into_output().await?,
            PortModeState::Enabled { pin, present: _ } => pin.try_into_output().await?,
        };
        pin.set_high().await.unwrap(); // TODO
        Ok(Self::Disabled { pin })
    }

    async fn enable(self) -> Result<Self, E> {
        let pin = match self {
            PortModeState::Disabled { pin } => pin.try_into_input().await?,
            PortModeState::Enabled { pin, present: _ } => pin.try_into_input().await?,
        };
        Ok(Self::Enabled {
            pin,
            present: false,
        })
    }
}

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy, Getters)]
pub struct HexpansionPortEvent {
    /// The port this event is about
    #[getset(get = "pub")]
    port: HexpansionPort,

    /// The time the event happened
    #[getset(get = "pub")]
    time: Instant,

    /// The state the port is now in
    #[getset(get = "pub")]
    state: HexpansionState,
}

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy)]
pub enum HexpansionState {
    /// The port is disabled and will not supply power
    Disabled,

    /// The port is enabled, but no hexpansion is demanding power from it
    Empty,

    /// The port is enabled and contains a hexpansion that is demanding power
    Occupied,
}
