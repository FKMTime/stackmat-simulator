#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::time::{Duration, Instant};
use esp_hal::uart::UartTx;
use esp_hal::{DriverMode, main};
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let serial_config = esp_hal::uart::Config::default().with_baudrate(1200);
    let Ok(mut uart) =
        UartTx::new(peripherals.UART1, serial_config).map(|u| u.with_tx(peripherals.GPIO20))
    else {
        log::error!("Stackmat task error while creating UartRx instance!");
        loop {}
    };

    let delay = Delay::new();

    send_timer_packet(
        &mut uart,
        &generate_timer_packet(StackmatTimerState::Reset, 0, 0, 0),
    );
    delay.delay_millis(2500);

    send_timer_packet(
        &mut uart,
        &generate_timer_packet(StackmatTimerState::Running, 0, 0, 0),
    );

    let start = Instant::now();
    loop {
        let time = ms_to_time(start.elapsed().as_millis());
        send_timer_packet(
            &mut uart,
            &generate_timer_packet(StackmatTimerState::Running, time.0, time.1, time.2),
        );
        delay.delay_millis(60);
    }
}

fn send_timer_packet<DM: DriverMode>(uart: &mut UartTx<'_, DM>, buf: &[u8]) {
    let mut offset = 0;

    loop {
        let res = UartTx::write(uart, &buf[offset..]);
        match res {
            Ok(n) => {
                if offset + n >= buf.len() {
                    break;
                }

                offset = n;
            }
            Err(e) => {
                log::error!("Uart::write_bytes error: {e:?}");
                break;
            }
        }
    }
}

fn generate_timer_packet(state: StackmatTimerState, minutes: u8, seconds: u8, ms: u16) -> [u8; 8] {
    let mut tmp = ['0' as u8; 8]; // fill with ascii '0'
    tmp[0] = state.to_u8();
    insert_digits(minutes as u64, &mut tmp[1..2]);
    insert_digits(seconds as u64, &mut tmp[2..4]);
    insert_digits(ms as u64, &mut tmp[4..7]);

    // sum of all digits + 64
    let sum = 64 + tmp[1..7].iter().map(|&x| x - '0' as u8).sum::<u8>();
    tmp[7] = sum;
    tmp
}

// insert digits into buffer as ascii bytes
fn insert_digits(mut nmb: u64, buf: &mut [u8]) {
    if buf.len() == 0 {
        return;
    }
    let mut offset = buf.len() - 1;

    loop {
        let dig = nmb % 10;
        buf[offset] = '0' as u8 + dig as u8;
        nmb /= 10;

        if offset == 0 {
            break;
        }
        offset -= 1;
    }
}

fn ms_to_time(ms: u64) -> (u8, u8, u16) {
    (
        (ms / 60000) as u8,
        ((ms % 60000) / 1000) as u8,
        (ms % 1000) as u16,
    )
}

#[allow(dead_code)]
enum StackmatTimerState {
    Unknown,
    Reset,
    Running,
    Stopped,
}

impl StackmatTimerState {
    fn to_u8(&self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Reset => b'I',
            Self::Running => b' ',
            Self::Stopped => b'S',
        }
    }
}
