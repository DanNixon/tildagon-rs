mod buttons;
mod gc9a01;
mod leds;

pub use buttons::*;
pub use gc9a01::*;
pub use leds::*;

use crate::front::{
    leds::PixelBuffer,
    variants::{FrontBoardDisplay, FrontBoardLeds},
};

pub struct Emf2024FrontBoard;

impl FrontBoardDisplay for Emf2024FrontBoard {
    type Display = Gc9a01;
}

impl FrontBoardLeds for Emf2024FrontBoard {
    const NUM_LEDS: usize = 19;
    const RMT_BUFFER_SIZE: usize = esp_hal_smartled::buffer_size(19);

    type Pixels = Pixel;
    type PixelBuffer = PixelBuffer<19>;
}
