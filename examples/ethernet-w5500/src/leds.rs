use chrono::{FixedOffset, Timelike};
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

        match crate::wall_time::now() {
            Some(time) => {
                let time = time.with_timezone(&FixedOffset::east_opt(60 * 60).unwrap());

                let hour_pixel_idx = (time.hour12().1 - 1) as usize;
                let minute_pixel_idx = match (time.minute() / 5) as usize {
                    0 => 11,
                    n => n - 1,
                };
                let second_pixel_idx = match (time.second() / 5) as usize {
                    0 => 11,
                    n => n - 1,
                };

                leds.front_pixels().fill(RGB8::default());
                leds.front_pixels()[hour_pixel_idx] =
                    leds.front_pixels()[hour_pixel_idx] + RGB8::new(127, 0, 0);
                leds.front_pixels()[minute_pixel_idx] =
                    leds.front_pixels()[minute_pixel_idx] + RGB8::new(0, 127, 0);
                leds.front_pixels()[second_pixel_idx] =
                    leds.front_pixels()[second_pixel_idx] + RGB8::new(0, 0, 127);
            }
            None => {
                leds.front_pixels().fill(RGB8::new(128, 0, 0));
            }
        }

        leds.write().unwrap();
    }
}
