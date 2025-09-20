pub mod config;
mod registers;

pub mod interface;
pub use interface::I2cAddr;
pub mod types;

#[allow(clippy::module_inception)]
mod bmi2;
pub use bmi2::Bmi2;
