use crate::front::{
    leds::{BaseBoardLed, PixelBuffer},
    variants::FrontBoardLeds,
};
use defmt::Format;
use smart_leds::RGB8;

pub struct NoFrontBoard;

impl FrontBoardLeds for NoFrontBoard {
    const NUM_LEDS: usize = 1;
    const RMT_BUFFER_SIZE: usize = esp_hal_smartled::buffer_size(1);

    type Pixels = Pixel;
    type PixelBuffer = PixelBuffer<1>;
}

#[derive(Debug, Format, PartialEq, Eq, Clone, Copy)]
#[repr(usize)]
pub enum Pixel {
    BaseBoard = 0,
}

impl BaseBoardLed for <NoFrontBoard as FrontBoardLeds>::PixelBuffer {
    fn base_board(&mut self) -> &mut RGB8 {
        self.pixel(Pixel::BaseBoard as usize).unwrap()
    }
}
