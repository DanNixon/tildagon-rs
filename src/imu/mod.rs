use crate::i2c::{BlockingI2cDeviceWrapper, SharedI2cDevice, SystemI2cBus};
use bmi2::{Bmi2, I2cAddr, config::BMI270_CONFIG_FILE, interface::I2cInterface, types::Burst};
use defmt::info;

// Re-export driver crate
pub use bmi2;

pub type I2cDevice = SharedI2cDevice<SystemI2cBus>;
pub type Imu = Bmi2<I2cInterface<BlockingI2cDeviceWrapper<I2cDevice>>, embassy_time::Delay, 256>;

pub async fn init(
    i2c: I2cDevice,
) -> Result<Imu, bmi2::types::Error<<I2cDevice as embedded_hal_async::i2c::ErrorType>::Error>> {
    let i2c = BlockingI2cDeviceWrapper::new(i2c);

    let mut imu = Bmi2::new_i2c(
        i2c,
        embassy_time::Delay,
        I2cAddr::Alternative,
        Burst::new(255),
    );

    info!("IMU chip ID: {}", imu.get_chip_id().unwrap());

    imu.init(&BMI270_CONFIG_FILE).unwrap();

    Ok(imu)
}
