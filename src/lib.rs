#![no_std]
#![feature(adt_const_params)]

pub mod hexpansion_slots;
pub mod i2c;
pub mod imu;
pub mod pins;
pub mod resources;
pub mod system;

#[cfg(not(feature = "top-board-none"))]
pub mod buttons;

#[cfg(not(feature = "top-board-none"))]
pub mod display;

pub mod leds;

// Re-exports
pub use esp_hal;

#[cfg(not(any(feature = "top-board-none", feature = "top-board-2024")))]
compile_error!("You must enable at least one `top-board-*` feature.");
