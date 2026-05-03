use chrono::{Datelike, FixedOffset, Timelike};
use core::fmt::Write;
use embassy_net::Stack;
use embassy_time::{Duration, Ticker};
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::{Dimensions, DrawTarget, Point, RgbColor},
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
    net: Stack<'static>,
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

        let mut time_string = heapless::String::<100>::new();
        match crate::wall_time::now() {
            Some(time) => {
                let time = time.with_timezone(&FixedOffset::east_opt(60 * 60).unwrap());

                time_string
                    .write_fmt(format_args!(
                        "{:04}-{:02}-{:02}\n{:02}:{:02}:UTC{:02}\n{}",
                        time.year(),
                        time.month(),
                        time.day(),
                        time.hour(),
                        time.minute(),
                        time.second(),
                        time.timezone(),
                    ))
                    .unwrap();
            }
            None => {
                time_string.write_str("unknown").unwrap();
            }
        }

        let mut network_string = heapless::String::<100>::new();
        network_string
            .write_fmt(format_args!(
                "Link: {}\nConfig: {}",
                net.is_link_up(),
                net.is_config_up()
            ))
            .unwrap();

        display.clear(Rgb565::BLACK).unwrap();
        TextBox::with_textbox_style(
            &time_string,
            Rectangle::with_center(centre + Point::new(0, 30), display.bounding_box().size),
            character_style,
            textbox_style,
        )
        .draw(&mut display)
        .unwrap();
        TextBox::with_textbox_style(
            &network_string,
            Rectangle::with_center(centre + Point::new(0, -40), display.bounding_box().size),
            character_style,
            textbox_style,
        )
        .draw(&mut display)
        .unwrap();
    }
}
