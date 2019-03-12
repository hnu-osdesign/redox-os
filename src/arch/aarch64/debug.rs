use core::fmt;
use spin::MutexGuard;

use devices::uart_pl011::SerialPort;
use super::device::serial::COM1;

pub struct Writer<'a> {
    serial: MutexGuard<'a, Option<SerialPort>>,
}

impl<'a> Writer<'a> {
    pub fn new() -> Writer<'a> {
        Writer {
            serial: unsafe { COM1.lock() },
        }
    }
}

impl<'a> fmt::Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        if let Some(ref mut serial_port) = *self.serial {
            serial_port.write_str(s);
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}
