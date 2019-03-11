//! # Page table
//! Code borrowed from [Phil Opp's Blog](http://os.phil-opp.com/modifying-page-tables.html)

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use memory::allocate_frames;

use super::entry::{EntryFlags, Entry};
use super::ENTRY_COUNT;

pub const P4: *mut Table<Level4> = (::RECURSIVE_PAGE_OFFSET | 0x7ffffff000) as *mut _;

pub trait TableLevel {}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L> Table<L> where L: TableLevel {
    pub fn is_unused(&self) -> bool {
        if self.entry_count() > 0 {
            return false;
        }

        true
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_zero();
        }
    }

    /// Set number of entries in first table entry
    fn set_entry_count(&mut self, count: u64) {
    }

    /// Get number of entries in first table entry
    fn entry_count(&self) -> u64 {
        0
    }

    pub fn increment_entry_count(&mut self) {
    }

    pub fn decrement_entry_count(&mut self) {
    }
}

impl<L> Table<L> where L: HierarchicalLevel {
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index).map(|address| unsafe { &*(address as *const _) })
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index).map(|address| unsafe { &mut *(address as *mut _) })
    }

    pub fn next_table_create(&mut self, index: usize) -> &mut Table<L::NextLevel> {
        self.next_table_mut(index).unwrap()
    }

    fn next_table_address(&self, index: usize) -> Option<usize> {
        None
    }
}

impl<L> Index<usize> for Table<L> where L: TableLevel {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}
