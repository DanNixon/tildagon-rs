#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::{Duration, Ticker, Timer};
use panic_rtt_target as _;
use smart_leds::RGB8;
use static_cell::StaticCell;
use tildagon::{
    esp_hal::{self, clock::CpuClock, rmt::Rmt, time::Rate, timer::systimer::SystemTimer},
    hexpansion_slots::{
        HexpansionSlot, HexpansionSlotControl, HexpansionSlotEvent, HexpansionState,
    },
    i2c::{SharedI2cBus, SharedI2cDevice, SystemI2cBus},
    leds::Leds,
    pins::{PinControl, async_digital::OutputPin},
    resources::*,
};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let config = tildagon::esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = tildagon::esp_hal::init(config);
    let r = tildagon::split_resources!(p);

    esp_alloc::heap_allocator!(size: 64 * 1024);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024);

    let timer0 = SystemTimer::new(p.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    static I2C_BUS: StaticCell<SharedI2cBus<tildagon::i2c::I2c>> = StaticCell::new();
    let (bus, _reset) = tildagon::i2c::i2c_bus(r.i2c).await;
    let i2c_bus = I2C_BUS.init(bus);

    static I2C_SYSTEM: StaticCell<SharedI2cBus<tildagon::i2c::SystemI2cBus>> = StaticCell::new();
    let i2c_system = I2C_SYSTEM.init(tildagon::i2c::system_i2c_bus(i2c_bus));

    let mut pin_control = PinControl::new(i2c_system);
    // pin_control.reset().unwrap();
    pin_control.init().await.unwrap();
    let pins = pin_control.pins();

    let mut usb_sel = pins
        .other
        .usb_select
        .into_output(SharedI2cDevice::new(i2c_system))
        .await
        .unwrap();
    usb_sel.set_low().await.unwrap();

    let mut hex_slots =
        HexpansionSlotControl::try_new(SharedI2cDevice::new(i2c_system), pins.hexpansion_detect)
            .await
            .unwrap();

    let rmt: Rmt<'_, esp_hal::Async> = Rmt::new(p.RMT, Rate::from_mhz(80)).unwrap().into_async();

    let mut leds = Leds::try_new(
        SharedI2cDevice::new(i2c_system),
        pins.led,
        r.led,
        rmt.channel0,
    )
    .await
    .unwrap();
    leds.set_power(true).await.unwrap();
    leds.intensity = 32;

    spawner.must_spawn(led_task(leds));

    // A little time for other tasks to start.
    // Hacky as all fuck but good enough for a demo.
    // Use channels to indicate readiness properly, mkay.
    Timer::after_millis(500).await;

    hex_slots
        .set_enabled(HexpansionSlot::A, false)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionSlot::B, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionSlot::C, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionSlot::D, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionSlot::E, true)
        .await
        .unwrap();
    hex_slots
        .set_enabled(HexpansionSlot::F, false)
        .await
        .unwrap();

    let mut tick = Ticker::every(Duration::from_millis(100));
    let mut hex_control_sub = HEX_CONTROL_CHANNEL.subscriber().unwrap();
    let event_pub = EVENT_CHANNEL.publisher().unwrap();

    loop {
        match select(tick.next(), hex_control_sub.next_message()).await {
            Either::First(_) => {
                let regs = pin_control.read_input_registers().await.unwrap();

                for event in hex_slots.update(&regs) {
                    info!("Hexpansion event: {}", event);
                    event_pub.publish(Event::HexpansionSlot(event)).await;
                }
            }
            Either::Second(WaitResult::Lagged(_)) => panic!(),
            Either::Second(WaitResult::Message(msg)) => {
                hex_slots.set_enabled(msg.slot, msg.enable).await.unwrap();
            }
        }
    }
}

#[derive(Clone)]
enum Event {
    HexpansionSlot(HexpansionSlotEvent),
}

static EVENT_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Event, 12, 4, 4> =
    PubSubChannel::new();

#[derive(Clone)]
struct HexpansionControlMsg {
    slot: HexpansionSlot,
    enable: bool,
}

static HEX_CONTROL_CHANNEL: PubSubChannel<CriticalSectionRawMutex, HexpansionControlMsg, 6, 1, 1> =
    PubSubChannel::new();

#[embassy_executor::task]
async fn led_task(mut leds: Leds<SharedI2cDevice<SystemI2cBus>>) {
    const HEX_DISABLED_COLOUR: RGB8 = RGB8::new(255, 0, 0);
    const HEX_EMPTY_COLOUR: RGB8 = RGB8::new(255, 192, 0);
    const HEX_OCCUPIED_COLOUR: RGB8 = RGB8::new(255, 255, 255);

    let mut event_sub = EVENT_CHANNEL.subscriber().unwrap();

    loop {
        match event_sub.next_message().await {
            WaitResult::Lagged(_) => panic!(),
            WaitResult::Message(Event::HexpansionSlot(event)) => {
                *leds.main_board_pixel() = match *event.state() {
                    HexpansionState::Disabled => HEX_DISABLED_COLOUR,
                    HexpansionState::Empty => HEX_EMPTY_COLOUR,
                    HexpansionState::Occupied => HEX_OCCUPIED_COLOUR,
                };
                leds.write().await.unwrap();

                Timer::after_millis(50).await;

                *leds.main_board_pixel() = RGB8::new(0, 0, 0);
                leds.write().await.unwrap();
            }
        }
    }
}
