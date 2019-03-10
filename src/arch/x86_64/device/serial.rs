use devices::uart_16550::SerialPort;
use syscall::io::Pio;
use spin::Mutex;

pub static COM1: Mutex<Option<SerialPort<Pio<u8>>>> = Mutex::new(Some(SerialPort::<Pio<u8>>::new(0x3F8)));
pub static COM2: Mutex<Option<SerialPort<Pio<u8>>>> = Mutex::new(Some(SerialPort::<Pio<u8>>::new(0x2F8)));

pub unsafe fn init() {
    if let Some(ref mut serial_port) = *COM1.lock() {
        serial_port.init();
    }
    if let Some(ref mut serial_port) = *COM2.lock() {
        serial_port.init();
    }
}
