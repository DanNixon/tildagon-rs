use crate::hexpansions::HexpansionPort;
use smart_leds::RGB8;

pub struct PixelBuffer<const N: usize>([RGB8; N]);

impl<const N: usize> Default for PixelBuffer<N> {
    fn default() -> Self {
        Self([RGB8::default(); N])
    }
}

impl<const N: usize> PixelBuffer<N> {
    pub fn pixel(&mut self, i: usize) -> Option<&mut RGB8> {
        self.0.get_mut(i)
    }

    pub fn pixels(&mut self, range: core::ops::Range<usize>) -> &mut [RGB8] {
        &mut self.0[range]
    }

    pub fn into_iter(&self) -> core::array::IntoIter<RGB8, N> {
        self.0.into_iter()
    }
}

pub trait BaseBoardLed {
    fn base_board(&mut self) -> &mut RGB8;
}

pub trait HexpansionPortLed {
    fn hexpansion_port(&mut self, port: HexpansionPort) -> &mut RGB8;
}

pub trait FrontLeds {
    fn front(&mut self) -> &mut [RGB8];
}
