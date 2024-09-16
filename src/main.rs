#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, delay::Delay, gpio::Io, peripherals::Peripherals, prelude::*,
    system::SystemControl, uart::Uart,
};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let serial_config = esp_hal::uart::config::Config::default().baudrate(1200);
    let mut serial = Uart::new_with_config(
        peripherals.UART1,
        serial_config,
        &clocks,
        io.pins.gpio4, // tx
        io.pins.gpio5,
    )
    .unwrap();

    esp_println::logger::init_logger_from_env();

    send_timer_packet(
        &mut serial,
        &generate_timer_packet(StackmatTimerState::Reset, 0, 0, 0),
    );
    delay.delay_millis(2500);

    send_timer_packet(
        &mut serial,
        &generate_timer_packet(StackmatTimerState::Running, 0, 0, 0),
    );

    let start = esp_hal::time::current_time();
    loop {
        let elapsed = esp_hal::time::current_time() - start;
        let time = ms_to_time(elapsed.to_millis());
        send_timer_packet(
            &mut serial,
            &generate_timer_packet(StackmatTimerState::Running, time.0, time.1, time.2),
        );
        delay.delay_millis(60);
    }
}

fn send_timer_packet<T: esp_hal::prelude::_esp_hal_uart_Instance, M: esp_hal::Mode>(
    uart: &mut Uart<T, M>,
    buf: &[u8],
) {
    let mut offset = 0;

    loop {
        let res = Uart::write_bytes(uart, &buf[offset..]);
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

fn generate_timer_packet(state: StackmatTimerState, minutes: u8, seconds: u8, ms: u16) -> [u8; 9] {
    let mut tmp = ['0' as u8; 9]; // fill with ascii '0'
    tmp[0] = state.to_u8();
    insert_digits(minutes as u64, &mut tmp[1..2]);
    insert_digits(seconds as u64, &mut tmp[2..4]);
    insert_digits(ms as u64, &mut tmp[4..7]);

    // sum of all digits + 64
    let sum = 64 + tmp[1..7].iter().map(|&x| x - '0' as u8).sum::<u8>();
    tmp[7] = sum;
    tmp[8] = b'\r';
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
