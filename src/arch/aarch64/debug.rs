use core::fmt;
use spin::MutexGuard;

use devices::uart_pl011::SerialPort;
use super::device::serial::COM1;

pub struct Writer<'a> {
    serial: MutexGuard<'a, SerialPort<Pio<u8>>>,
}

impl<'a> Writer<'a> {
    pub fn new() -> Writer<'a> {
        Writer {
            serial: COM1.lock(),
        }
    }
}

impl<'a> fmt::Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.serial.write_str(s)
    }
}
