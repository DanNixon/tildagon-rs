//! Front board specific driver collections.
//!
//! Drivers are added under the first board that introduced that hardware, subsequent reuse on later boards can just reimport as required.

pub mod emf2024;
pub mod none;

pub use emf2024::Emf2024FrontBoard;
pub use none::NoFrontBoard;

use crate::button_collection::ButtonCollection;

#[derive(Debug, defmt::Format)]
pub enum FrontBoard {
    None,
    TwentyTwentyFour,
    TwentyTwentySix,
}

pub trait FrontBoardDisplay {
    type Display;
}

pub trait FrontBoardLeds {
    const NUM_LEDS: usize;
    const RMT_BUFFER_SIZE: usize;

    type Pixels;
    type PixelBuffer;
}

pub trait FrontBoardButtons<B, I2C, const N: usize> {
    type Buttons = B;
    type ButtonCollection = ButtonCollection<B, I2C, N>;
}
