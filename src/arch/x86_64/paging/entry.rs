//! # Page table entry
//! Some code borrowed from [Phil Opp's Blog](http://os.phil-opp.com/modifying-page-tables.html)

use crate::memory::Frame;

use super::PhysicalAddress;

//存储页表条目信息的结构
#[repr(packed(8))]
//repr(packed(8))这是是强制 Rust 不填充空数据，各个类型的数据紧密排列。这样有助于提升内存的使用效率，但很可能会导致其他的副作用。
pub struct Entry(u64);

bitflags! {
    pub struct EntryFlags: u64 {//64位
        const PRESENT =         1;//present位为1，表示对应的页或者页表已经载入到内存中。如果为0，对其访问，则会发生缺页异常
        const WRITABLE =        1 << 1;//允许写入页面
        const USER_ACCESSIBLE = 1 << 2;//用户访问权限，如果没有设置，则只能在内核模式下访问此页面
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;//是否禁用缓存
        const ACCESSED =        1 << 5;//该页是否使用过
        const DIRTY =           1 << 6;//swap进程可以通过这个位来决定是否选择这个页面进行交换
        const HUGE_PAGE =       1 << 7;//决定是使用4k的页还是4M的大页
        const GLOBAL =          1 << 8;//全局设定，页面是否在所有地址空间中都可用
        const NO_EXECUTE =      1 << 63;//禁止在此页面执行代码
        /*
        9-11位 OS自由分配使用
        12-51位 物理地址  40位
        52-62位 OS自由分配使用
        */
    }
}

pub const ADDRESS_MASK: usize = 0x000f_ffff_ffff_f000;
pub const COUNTER_MASK: u64 = 0x3ff0_0000_0000_0000;
//ADDRESS_MASK可以用来获取地址；COUNTER_MASK 可用来获取计数值

impl Entry {
    //清除条目
    pub fn set_zero(&mut self) {
        self.0 = 0;
    }

    //检测该页表条目是否可以用
    pub fn is_unused(&self) -> bool {
        self.0 == (self.0 & COUNTER_MASK)
        /*COUNTER_MASK  0x3ff0_0000_0000_0000
        有效：self.0== 0x3ff0_0000_0000_0000
        */
    }

    //使页表条目不可用
    pub fn set_unused(&mut self) {
        self.0 &= COUNTER_MASK;
    }

    //获取此页的地址
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.0 as usize & ADDRESS_MASK)
        //和ADDRESS_MASK相与获取地址
    }

    //获取当前页表条目的标志位flags
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
        //from_bits_truncate 截断，删除和该标志不对应的任何位
    }

    //获取关联帧
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {//检测present是否在EntryFlags中
            Some(Frame::containing_address(self.address()))//创建帧
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        debug_assert!(frame.start_address().get() & !ADDRESS_MASK == 0);
        //assert!是用于断言布尔表达式是否为true，带debug说明只能在调试模式下使用
        //start_address()获取帧的地址
        //判断地址是否超出范围
        self.0 = (frame.start_address().get() as u64) | flags.bits() | (self.0 & COUNTER_MASK);
        //将页表条目的信息拼接起来，物理地址（12-51位）|页表条目管理信息(低12位）|计算值（52-62位）
    }

    //获取条目中的第52-61位，用作页表的计数器  10位
    pub fn counter_bits(&self) -> u64 {
        (self.0 & COUNTER_MASK) >> 52  //获取counter
    }

    //在条目中设置位52-61，用作页表的计数器
    pub fn set_counter_bits(&mut self, count: u64) {
        self.0 = (self.0 & !COUNTER_MASK) | (count << 52);
        //首先(self.0 & !COUNTER_MASK)将52-62位的数据清0，接着将要设置的值count移动到52-62位后和(self.0 & !COUNTER_MASK)拼接。
    }
}

/*
用来验证非测试代码是否按照期望的方式运行的
当使用 cargo test 命令运行测试时，Rust 会构建一个测试执行程序用来调用标记了 test 属性的函数，并报告每一个测试是通过还是失败。
*/
#[cfg(test)]
mod tests {
    #[test]
    fn entry_has_required_arch_alignment() {
        use super::Entry;
        assert!(core::mem::align_of::<Entry>() >= core::mem::align_of::<u64>(), "alignment of Entry is less than the required alignment of u64 ({} < {})", core::mem::align_of::<Entry>(), core::mem::align_of::<u64>());
    }
}
