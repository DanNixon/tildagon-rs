pub struct BlockingI2cDeviceWrapper<I2C> {
    inner: I2C,
}

impl<I2C> BlockingI2cDeviceWrapper<I2C> {
    pub fn new(inner: I2C) -> Self {
        Self { inner }
    }
}

impl<I2C: embedded_hal_async::i2c::ErrorType> embedded_hal::i2c::ErrorType
    for BlockingI2cDeviceWrapper<I2C>
{
    type Error = I2C::Error;
}

impl<I2C: embedded_hal_async::i2c::I2c> embedded_hal::i2c::I2c for BlockingI2cDeviceWrapper<I2C> {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        embassy_futures::block_on(async { self.inner.transaction(address, operations).await })
    }
}
