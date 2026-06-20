#![no_std]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(associated_type_defaults)]
#![feature(inherent_associated_types)]

pub mod button_collection;
pub mod front;
pub mod hexpansions;
pub mod i2c;
pub mod imu;
pub mod pins;
pub mod resources;
pub mod system;

// Re-exports
pub use bmi2;
pub use bq25895;
pub use embedded_aw9523;
pub use esp_hal;
pub use esp_hal_smartled;
pub use smart_leds;
