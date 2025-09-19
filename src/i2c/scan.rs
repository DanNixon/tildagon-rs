use super::SharedI2cBus;
use defmt::{Format, info};
use embedded_hal_async::i2c::I2c;
use heapless::Vec;

#[derive(Debug)]
pub struct ScanResult<E> {
    address: u8,
    result: Result<(), E>,
}

pub type ScanResults<E> = Vec<ScanResult<E>, 128>;

pub async fn scan_bus<BUS, E>(bus: &SharedI2cBus<BUS>) -> ScanResults<E>
where
    BUS: I2c<Error = E>,
    E: core::fmt::Debug,
{
    let mut results = ScanResults::new();

    let mut bus = bus.lock().await;

    for address in 0..127 {
        let result = bus.read(address, &mut [0]).await;
        results.push(ScanResult { address, result }).unwrap();
    }

    results
}

pub fn report_present_devices<E>(results: &ScanResults<E>) {
    info!("Device(s) found at:");
    for result in results.iter().filter(|r| r.result.is_ok()) {
        info!(" + 0x{:02X}", result.address);
    }
}

pub fn report_absent_devices<E: Format>(results: &ScanResults<E>) {
    info!("No device(s) found at:");
    for result in results.iter().filter(|r| r.result.is_err()) {
        info!(" - 0x{:02X}: {}", result.address, result.result);
    }
}
