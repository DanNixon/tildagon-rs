use embassy_time::{Duration, Ticker};
use smart_leds::RGB8;
use tildagon::{
    i2c::{SharedI2cDevice, SystemI2cBus},
    leds::Leds,
};

#[embassy_executor::task]
pub(super) async fn task(mut leds: Leds<'static, SharedI2cDevice<SystemI2cBus>>) {
    *leds.main_board_pixel() = RGB8::new(128, 0, 128);

    leds.write().unwrap();

    let mut tick = Ticker::every(Duration::from_hz(1));

    loop {
        tick.next().await;

        // TODO
    }
}
