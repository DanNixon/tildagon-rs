#[cfg(not(feature = "top-board-none"))]
use crate::hexpansion_slots::HexpansionSlot;
use crate::{pins::LedPins, pins::async_digital::OutputPin, resources::LedResources};
use defmt::Format;
use embedded_hal::digital::{ErrorKind, PinState};
use esp_hal::{
    Blocking,
    rmt::{Channel, ChannelCreator},
};
use esp_hal_smartled::{LedAdapterError, SmartLedsAdapter, buffer_size_async};
use smart_leds::{RGB8, SmartLedsWrite, brightness, gamma};

pub struct Leds<I2C> {
    led_power: crate::pins::OutputPin<I2C>,
    leds: SmartLedsAdapter<Channel<Blocking, 0>, { LED_COUNT * 25 }>,

    pub intensity: u8,
    pub pixels: [RGB8; LED_COUNT],
}

impl<I2C, E> Leds<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    pub async fn try_new(
        i2c: I2C,
        pins: LedPins,
        r: LedResources<'static>,
        rmt_ch: ChannelCreator<Blocking, 0>,
    ) -> Result<Self, E> {
        let buffer = [0_u32; buffer_size_async(LED_COUNT)];
        let leds = SmartLedsAdapter::new(rmt_ch, r.data, buffer);

        Ok(Self {
            led_power: pins.power_enable.into_output(i2c).await?,
            leds,
            intensity: 255,
            pixels: [RGB8::default(); LED_COUNT],
        })
    }

    pub async fn set_power(&mut self, on: bool) -> Result<(), ErrorKind> {
        self.led_power
            .set_state(match on {
                true => PinState::High,
                false => PinState::Low,
            })
            .await
    }

    pub fn write(&mut self) -> Result<(), LedAdapterError> {
        self.leds
            .write(brightness(gamma(self.pixels.into_iter()), self.intensity))
    }

    pub fn main_board_pixel(&mut self) -> &mut RGB8 {
        &mut self.pixels[0]
    }

    #[cfg(feature = "top-board-2024")]
    pub fn hexpansion_pixel(&mut self, hex: HexpansionSlot) -> &mut RGB8 {
        let pixel = match hex {
            HexpansionSlot::A => Pixel::HexpansionA,
            HexpansionSlot::B => Pixel::HexpansionB,
            HexpansionSlot::C => Pixel::HexpansionC,
            HexpansionSlot::D => Pixel::HexpansionD,
            HexpansionSlot::E => Pixel::HexpansionE,
            HexpansionSlot::F => Pixel::HexpansionF,
        };
        &mut self.pixels[pixel as usize]
    }

    #[cfg(feature = "top-board-2024")]
    pub fn hexpansion_pixels(&mut self) -> &mut [RGB8] {
        &mut self.pixels[13..19]
    }

    #[cfg(feature = "top-board-2024")]
    pub fn front_pixels(&mut self) -> &mut [RGB8] {
        &mut self.pixels[1..13]
    }
}

#[cfg(feature = "top-board-none")]
pub const LED_COUNT: usize = 1;

#[cfg(feature = "top-board-none")]
#[derive(Debug, Format, PartialEq, Eq, Clone)]
#[repr(usize)]
pub enum Pixel {
    BaseBoard = 0,
}

#[cfg(feature = "top-board-2024")]
pub const LED_COUNT: usize = 19;

#[cfg(feature = "top-board-2024")]
#[derive(Debug, Format, PartialEq, Eq, Clone)]
#[repr(usize)]
pub enum Pixel {
    BaseBoard = 0,
    Front1 = 1,
    Front2 = 2,
    Front3 = 3,
    Front4 = 4,
    Front5 = 5,
    Front6 = 6,
    Front7 = 7,
    Front8 = 8,
    Front9 = 9,
    Front10 = 10,
    Front11 = 11,
    Front12 = 12,
    HexpansionA = 13,
    HexpansionB = 14,
    HexpansionC = 15,
    HexpansionD = 16,
    HexpansionE = 17,
    HexpansionF = 18,
}
