//! # Paging
//! Some code was borrowed from [Phil Opp's Blog](http://os.phil-opp.com/modifying-page-tables.html)

use core::{mem, ptr};
use core::ops::{Deref, DerefMut};

use memory::{allocate_frames, Frame};

use self::entry::EntryFlags;
use self::mapper::Mapper;
use self::temporary_page::TemporaryPage;

pub mod entry;
pub mod mapper;
pub mod table;
pub mod temporary_page;

/// Number of entries per page table
pub const ENTRY_COUNT: usize = 512;

/// Size of pages
pub const PAGE_SIZE: usize = 4096;

/// Copy tdata, clear tbss, set TCB self pointer
unsafe fn init_tcb(cpu_id: usize) -> usize {
    0
}

/// Initialize paging
///
/// Returns page table and thread control block offset
pub unsafe fn init(cpu_id: usize, kernel_start: usize, kernel_end: usize, stack_start: usize, stack_end: usize) -> (ActivePageTable, usize) {
    extern {
        /// The starting byte of the text (code) data segment.
        static mut __text_start: u8;
        /// The ending byte of the text (code) data segment.
        static mut __text_end: u8;
        /// The starting byte of the _.rodata_ (read-only data) segment.
        static mut __rodata_start: u8;
        /// The ending byte of the _.rodata_ (read-only data) segment.
        static mut __rodata_end: u8;
        /// The starting byte of the _.data_ segment.
        static mut __data_start: u8;
        /// The ending byte of the _.data_ segment.
        static mut __data_end: u8;
        /// The starting byte of the thread data segment
        static mut __tdata_start: u8;
        /// The ending byte of the thread data segment
        static mut __tdata_end: u8;
        /// The starting byte of the thread BSS segment
        static mut __tbss_start: u8;
        /// The ending byte of the thread BSS segment
        static mut __tbss_end: u8;
        /// The starting byte of the _.bss_ (uninitialized data) segment.
        static mut __bss_start: u8;
        /// The ending byte of the _.bss_ (uninitialized data) segment.
        static mut __bss_end: u8;
    }

    (ActivePageTable::new(), 0)
}

pub unsafe fn init_ap(cpu_id: usize, bsp_table: usize, stack_start: usize, stack_end: usize) -> usize {
    extern {
        /// The starting byte of the thread data segment
        static mut __tdata_start: u8;
        /// The ending byte of the thread data segment
        static mut __tdata_end: u8;
        /// The starting byte of the thread BSS segment
        static mut __tbss_start: u8;
        /// The ending byte of the thread BSS segment
        static mut __tbss_end: u8;
    }
    0
}

pub struct ActivePageTable {
    mapper: Mapper,
}

pub enum PageTableType {
    User,
    Kernel
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(
                          PhysicalAddress::new(0)
                          ),
        };
        old_table
    }

    pub fn flush(&mut self, page: Page) {
    }

    pub fn flush_all(&mut self) {
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut TemporaryPage, f: F)
        where F: FnOnce(&mut Mapper)
    {
    }

    pub unsafe fn address(&self) -> usize {
        0
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable {
        InactivePageTable { p4_frame: frame }
    }

    pub unsafe fn from_address(cr3: usize) -> InactivePageTable {
        InactivePageTable { p4_frame: Frame::containing_address(PhysicalAddress::new(cr3)) }
    }

    pub unsafe fn address(&self) -> usize {
        0
    }
}

/// A physical address.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    pub fn new(address: usize) -> Self {
        PhysicalAddress(address)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

/// A virtual address.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    pub fn new(address: usize) -> Self {
        VirtualAddress(address)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

/// Page
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    number: usize
}

impl Page {
    pub fn start_address(&self) -> VirtualAddress {
        VirtualAddress::new(0)
    }

    pub fn p4_index(&self) -> usize {
        0
    }

    pub fn p3_index(&self) -> usize {
        0
    }

    pub fn p2_index(&self) -> usize {
        0
    }

    pub fn p1_index(&self) -> usize {
        0
    }

    pub fn containing_address(address: VirtualAddress) -> Page {
        Page { number: address.get() / PAGE_SIZE }
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }
}

pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}
