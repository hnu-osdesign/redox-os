use devices::uart_16550::SerialPort;
use syscall::io::Pio;
use spin::Mutex;

pub static COM1: Mutex<SerialPort<Pio<u8>>> = Mutex::new(SerialPort::<Pio<u8>>::new(0x3F8));

pub unsafe fn init() {
}
