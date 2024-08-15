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

    let config = esp_hal::uart::config::Config::default().baudrate(1200);

    let mut serial = Uart::new_with_config(
        peripherals.UART1,
        config,
        &clocks,
        None,
        io.pins.gpio4,
        io.pins.gpio5,
    )
    .unwrap();

    esp_println::logger::init_logger_from_env();

    loop {
        serial.write_byte(0x42).ok();
        let read = nb::block!(serial.read_byte());

        match read {
            Ok(read) => log::info!("Read 0x{:02x}", read),
            Err(err) => log::info!("Error {:?}", err),
        }

        delay.delay_millis(250);
    }
}
