use chrono::{Datelike, FixedOffset, Timelike};
use core::fmt::Write;
use embassy_time::{Duration, Ticker};
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::{Dimensions, DrawTarget, RgbColor},
    primitives::Rectangle,
};
use embedded_text::{
    TextBox,
    alignment::{HorizontalAlignment, VerticalAlignment},
    style::TextBoxStyleBuilder,
};
use tildagon::resources::{DisplayResources, TopBoardResources};

#[embassy_executor::task]
pub(super) async fn task(
    top_board: TopBoardResources<'static>,
    display: DisplayResources<'static>,
) {
    let mut display_buffer = [0_u8; 512];
    let mut display = tildagon::display::init(top_board, display, &mut display_buffer);
    display.clear(Rgb565::BLACK).unwrap();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Middle)
        .build();

    let centre = display.bounding_box().center();

    let mut tick = Ticker::every(Duration::from_hz(1));

    loop {
        tick.next().await;

        let mut str = heapless::String::<100>::new();
        match crate::wall_time::now() {
            Some(time) => {
                let time = time.with_timezone(&FixedOffset::east_opt(60 * 60).unwrap());

                str.write_fmt(format_args!(
                    "{:04}-{:02}-{:02}\n{:02}:{:02}:{:02}",
                    time.year(),
                    time.month(),
                    time.day(),
                    time.hour(),
                    time.minute(),
                    time.second()
                ))
                .unwrap();
            }
            None => {
                str.write_str("xxxx-xx-xx\nxx:xx:xx").unwrap();
            }
        }

        display.clear(Rgb565::BLACK).unwrap();
        TextBox::with_textbox_style(
            &str,
            Rectangle::with_center(centre, display.bounding_box().size),
            character_style,
            textbox_style,
        )
        .draw(&mut display)
        .unwrap();
    }
}
