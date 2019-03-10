use core::fmt;
use spin::MutexGuard;

use devices::uart_16550::SerialPort;
use syscall::io::Pio;

use super::device::serial::COM1;
#[cfg(feature = "graphical_debug")]
use super::graphical_debug::{DEBUG_DISPLAY, DebugDisplay};

pub struct Writer<'a> {
    serial: MutexGuard<'a, Option<SerialPort<Pio<u8>>>>,
    #[cfg(feature = "graphical_debug")]
    display: MutexGuard<'a, Option<DebugDisplay>>
}

impl<'a> Writer<'a> {
    pub fn new() -> Writer<'a> {
        Writer {
            serial: COM1.lock(),
            #[cfg(feature = "graphical_debug")]
            display: DEBUG_DISPLAY.lock(),
        }
    }
}

impl<'a> fmt::Write for Writer<'a> {
    #[cfg(not(feature = "graphical_debug"))]
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        if let Some(ref mut serial_port) = *self.serial {
            serial_port.write_str(s);
            Ok(())
        }
        else {
            Err(fmt::Error)
        }
    }

    #[cfg(feature = "graphical_debug")]
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        if let Some(ref mut display) = *self.display {
            let _ = display.write_str(s);
        }

        self.serial.write_str(s)
    }
}
