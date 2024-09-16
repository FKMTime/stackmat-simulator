#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::Io,
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    uart::{Uart, UartRx},
};
use nb::block;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let serial_config = esp_hal::uart::config::Config::default().baudrate(1200);
    let mut serial =
        UartRx::new_with_config(peripherals.UART1, serial_config, &clocks, io.pins.gpio20).unwrap();

    esp_println::logger::init_logger_from_env();

    let mut read_buf = heapless::Vec::<u8, 8>::new();
    loop {
        let read: u8 = block!(serial.read_byte()).unwrap();
        log::info!("read: {read}");
        if read == 0 || read == b'\r' || read_buf.len() == 8 {
            log::info!("BUF: {:?}", core::str::from_utf8(&read_buf));
            read_buf.clear();
        } else {
            _ = read_buf.push(read);
        }

        //delay.delay_millis(60);
    }
}
