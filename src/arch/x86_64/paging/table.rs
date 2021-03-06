//! # Page table
//! Code borrowed from [Phil Opp's Blog](http://os.phil-opp.com/modifying-page-tables.html)

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use crate::memory::allocate_frames;//分配一定数量的帧

use super::entry::{EntryFlags, Entry};
use super::ENTRY_COUNT;//定义在mod.rs 中，大小为512

pub const P4: *mut Table<Level4> = (crate::RECURSIVE_PAGE_OFFSET | 0x7f_ffff_f000) as *mut _;
//RECURSIVE_PAGE_OFFSET：usize = (-(PML4_SIZE as isize)) as usize
//PML4_SIZE: usize = 0x0000_0080_0000_0000
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

#[repr(packed(4096))]
pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],//512个类型为Entry的数组
    level: PhantomData<L>,
}
impl<L> Table<L> where L: TableLevel {
    //检测是否可用
    pub fn is_unused(&self) -> bool {
        if self.entry_count() > 0 {
            return false;
        }

        true
    }
    //将所有条目设置为未使用项，在创建新的页面表的时候用到
    pub fn zero(&mut self) {
        for entry in unsafe { &mut self.entries }.iter_mut() {
            entry.set_zero();//clear entry
        }
    }

    //设置第1个表项的页表条目数
    fn set_entry_count(&mut self, count: u64) {
        debug_assert!(count <= ENTRY_COUNT as u64, "count can't be greater than ENTRY_COUNT");
        unsafe { &mut self.entries[0] }.set_counter_bits(count)
    }

    ///获取第1个表项的页表条目数
    fn entry_count(&self) -> u64 {
        unsafe { &self.entries[0] }.counter_bits()  //获取计数值
    }
    //增加页表条目数
    pub fn increment_entry_count(&mut self) {
        let current_count = self.entry_count();
        self.set_entry_count(current_count + 1);
    }
    //减少页表条目数
    pub fn decrement_entry_count(&mut self) {
        let current_count = self.entry_count();
        self.set_entry_count(current_count - 1);
    }
}

impl<L> Table<L> where L: HierarchicalLevel {
    //将地址转化为引用 ：函数1和函数2
    //函数1
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index).map(|address| unsafe { &*(address as *const _) })
    }
    //函数2
    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index).map(|address| unsafe { &mut *(address as *mut _) })
    }

    pub fn next_table_create(&mut self, index: usize) -> &mut Table<L::NextLevel> {
        if self.next_table(index).is_none() {
            assert!(!self[index].flags().contains(EntryFlags::HUGE_PAGE),
                    "next_table_create does not support huge pages");
            let frame = allocate_frames(1).expect("no frames available");
            self.increment_entry_count();
            self[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE /* Allow users to go down the page table, implement permissions at the page level */);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
    }

    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();//取出对应的标志位
        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE) {
            //存在相应条目且是页大小为4KB 
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))//构造下一级索引的地址
        } else {
            None
        }
    }
}

impl<L> Index<usize> for Table<L> where L: TableLevel {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        unsafe { &self.entries[index] }//取出索引为index下的条目
    }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        unsafe { &mut self.entries[index] }
    }
}
