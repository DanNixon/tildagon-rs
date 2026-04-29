use chrono::{DateTime, Utc};
use core::{net::SocketAddr, sync::atomic::Ordering};
use defmt::{error, info};
use embassy_net::{
    Stack,
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_time::{Duration, Instant, Timer};
use portable_atomic::AtomicI64;
use sntpc::{NtpContext, NtpTimestampGenerator};
use sntpc_net_embassy::UdpSocketWrapper;

/// Offset in microseconds to add to uptime to get world time.
static BOOT_CLOCK_WALL_OFFSET_US: AtomicI64 = AtomicI64::new(0);

/// Gets the current wall time.
///
/// Time is only valid after a successful NTP sync.
pub(crate) fn now() -> Option<DateTime<Utc>> {
    let now = Instant::now().as_micros() as i64;
    let offset = BOOT_CLOCK_WALL_OFFSET_US.load(Ordering::Relaxed);
    match offset {
        0 => None,
        _ => Some(DateTime::from_timestamp_micros(now + offset).unwrap()),
    }
}

/// Name of the time server to use for NTP sync.
const TIME_SERVER: &str = "time.cloudflare.com";

#[embassy_executor::task]
pub(crate) async fn ntp_task(stack: Stack<'static>) -> ! {
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];

    let mut interval = Duration::from_secs(10);

    loop {
        let mut socket = UdpSocket::new(
            stack,
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );
        socket.bind(123).unwrap();
        let socket = UdpSocketWrapper::new(socket);

        let context = NtpContext::new(TimestampGen::default());

        match stack.dns_query(TIME_SERVER, DnsQueryType::A).await {
            Ok(ntp_addrs) => match ntp_addrs.first() {
                Some(addr) => {
                    let result =
                        sntpc::get_time(SocketAddr::from((*addr, 123)), &socket, context).await;

                    if let Ok(result) = result {
                        BOOT_CLOCK_WALL_OFFSET_US.add(result.offset, Ordering::Relaxed);
                        info!("Time synced: offset={}us {}", result.offset, now());

                        if result.offset.abs() < 10_000 {
                            interval = core::cmp::min(interval * 2, Duration::from_secs(300));
                        }
                    }
                }
                None => {
                    error!("DNS query returned no addresses");
                }
            },
            Err(e) => {
                error!("DNS query failed: {}", e);
            }
        }

        // Wait until it is time for the next sync
        info!("Waiting {}s for next sync", interval.as_secs());
        Timer::after(interval).await;
    }
}

#[derive(Copy, Clone, Default)]
struct TimestampGen {
    wall_time: DateTime<Utc>,
}

impl NtpTimestampGenerator for TimestampGen {
    fn init(&mut self) {
        self.wall_time = now().unwrap_or_default();
    }

    fn timestamp_sec(&self) -> u64 {
        self.wall_time.timestamp() as u64
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        self.wall_time.timestamp_subsec_micros()
    }
}
