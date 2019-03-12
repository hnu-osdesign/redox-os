use devices::uart_pl011::SerialPort;
use core::sync::atomic::{Ordering};
use init::device_tree;
use memory::Frame;
use paging::mapper::{MapperFlushAll, MapperType};
use paging::{ActivePageTable, Page, PageTableType, PhysicalAddress, VirtualAddress};
use paging::entry::EntryFlags;
use spin::Mutex;

pub static COM1: Mutex<SerialPort<Pio<u8>>> = Mutex::new(SerialPort::<Pio<u8>>::new(0x3F8));

pub unsafe fn init() {
}
