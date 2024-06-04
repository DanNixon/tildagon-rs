use crate::resources::{DisplayResources, TopBoardResources};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::{
    Blocking,
    dma::{DmaRxBuf, DmaTxBuf},
    gpio::{Level, Output},
    spi::{
        Mode,
        master::{Config, Spi, SpiDmaBus},
    },
    time::Rate,
};
use mipidsi::{
    NoResetPin,
    interface::SpiInterface,
    models::GC9A01,
    options::{ColorInversion, ColorOrder, Orientation, Rotation},
};

#[cfg(feature = "top-board-2024")]
type Display<'a> = mipidsi::Display<
    SpiInterface<
        'a,
        ExclusiveDevice<SpiDmaBus<'static, Blocking>, Output<'static>, NoDelay>,
        Output<'static>,
    >,
    GC9A01,
    NoResetPin,
>;

pub fn init<'a>(
    top_board: TopBoardResources<'static>,
    display: DisplayResources<'static>,
    buffer: &'a mut [u8],
) -> Display<'a> {
    #[allow(clippy::manual_div_ceil)]
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = esp_hal::dma_buffers!(32000);

    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi = Spi::new(
        display.spi,
        Config::default()
            .with_frequency(Rate::from_mhz(80))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(top_board.hs_1)
    .with_mosi(top_board.hs_2)
    .with_dma(display.dma)
    .with_buffers(dma_rx_buf, dma_tx_buf);

    let cs = Output::new(top_board.hs_4, Level::High, Default::default());
    let dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let dc = Output::new(top_board.hs_3, Level::High, Default::default());
    let di = SpiInterface::new(dev, dc, buffer);

    mipidsi::Builder::new(GC9A01, di)
        .display_size(240, 240)
        .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg180))
        .init(&mut embassy_time::Delay)
        .unwrap()
}
