use super::Emf2024FrontBoard;
use crate::{
    front::{
        leds::{BaseBoardLed, FrontLeds, HexpansionPortLed},
        variants::FrontBoardLeds,
    },
    hexpansions::HexpansionSlot,
};
use defmt::Format;
use smart_leds::RGB8;

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy)]
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

impl BaseBoardLed for <Emf2024FrontBoard as FrontBoardLeds>::PixelBuffer {
    fn base_board(&mut self) -> &mut RGB8 {
        self.pixel(Pixel::BaseBoard as usize).unwrap()
    }
}

impl HexpansionPortLed for <Emf2024FrontBoard as FrontBoardLeds>::PixelBuffer {
    fn hexpansion_port(&mut self, port: HexpansionSlot) -> &mut RGB8 {
        let pixel = match port {
            HexpansionSlot::A => Pixel::HexpansionA,
            HexpansionSlot::B => Pixel::HexpansionB,
            HexpansionSlot::C => Pixel::HexpansionC,
            HexpansionSlot::D => Pixel::HexpansionD,
            HexpansionSlot::E => Pixel::HexpansionE,
            HexpansionSlot::F => Pixel::HexpansionF,
        };
        self.pixel(pixel as usize).unwrap()
    }
}

impl FrontLeds for <Emf2024FrontBoard as FrontBoardLeds>::PixelBuffer {
    fn front(&mut self) -> &mut [RGB8] {
        self.pixels(1..13)
    }
}
