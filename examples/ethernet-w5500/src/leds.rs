use crate::LedResources;
use chrono::{FixedOffset, Timelike};
use embassy_time::{Duration, Ticker};
use tildagon::{
    esp_hal::{
        self,
        rmt::{ChannelCreator, PulseCode},
    },
    front::{
        FrontBoardLeds,
        leds::{BaseBoardLed, FrontLeds},
    },
    smart_leds::{RGB8, SmartLedsWrite},
};

#[embassy_executor::task]
pub(super) async fn task(
    r: LedResources<'static>,
    rmt_channel: ChannelCreator<'static, esp_hal::Blocking, 0>,
) {
    let mut rmt_buffer =
        [PulseCode::end_marker(); tildagon::front::Emf2024FrontBoard::RMT_BUFFER_SIZE];

    let mut adapter =
        tildagon::esp_hal_smartled::SmartLedsAdapter::new(rmt_channel, r.data, &mut rmt_buffer);

    let mut leds = <tildagon::front::Emf2024FrontBoard as FrontBoardLeds>::PixelBuffer::default();

    *leds.base_board() = RGB8::new(128, 0, 128);
    adapter.write(leds.into_iter()).unwrap();

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

                leds.front().fill(RGB8::default());
                leds.front()[hour_pixel_idx] = leds.front()[hour_pixel_idx] + RGB8::new(127, 0, 0);
                leds.front()[minute_pixel_idx] =
                    leds.front()[minute_pixel_idx] + RGB8::new(0, 127, 0);
                leds.front()[second_pixel_idx] =
                    leds.front()[second_pixel_idx] + RGB8::new(0, 0, 127);
            }
            None => {
                leds.front().fill(RGB8::new(128, 0, 0));
            }
        }

        adapter.write(leds.into_iter()).unwrap();
    }
}
