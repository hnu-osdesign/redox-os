use core::num::NonZeroU8;
use core::sync::atomic::{AtomicU64, Ordering};
use core::mem;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;

use x86::segmentation::Descriptor as X86IdtEntry;
use x86::dtables::{self, DescriptorTablePointer};

use crate::interrupt::*;
use crate::ipi::IpiKind;

use spin::RwLock;
/*pub enum Ordering {
    Relaxed,
    Release,
    Acquire,
    AcqRel,
    SeqCst,
}*/

pub static mut INIT_IDTR: DescriptorTablePointer<X86IdtEntry> = DescriptorTablePointer {//IDTR初始化
    limit: 0,//上限 16位
    base: 0 as *const X86IdtEntry//基址 64位
};//可变

#[thread_local]
pub static  IDTR: DescriptorTablePointer<X86IdtEntry> = DescriptorTablePointer {
    limit: 0,
    base: 0 as *const X86IdtEntry
};//不可变

pub type IdtEntries = [IdtEntry; 256];//256个中断
pub type IdtReservations = [AtomicU64; 4];//原子整数数组 ，u64

#[repr(packed)]
pub struct Idt {
    entries: IdtEntries,
    reservations: IdtReservations,
}
/*const fn new_idt_reservations() -> [AtomicU64; 4] {
    [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)] //
}
pub const fn new(v: u64) -> Self 创造一个原子整数
*/
impl Idt {
    pub const fn new() -> Self {//初始化
        Self {
            entries: [IdtEntry::new(); 256],
            reservations: new_idt_reservations(),
        }
    }
    #[inline]
    pub fn is_reserved(&self, index: u8) -> bool {
        let byte_index = index / 64;//右移6位，结果一定在0-3
        let bit = index % 64;

        unsafe { &self.reservations[usize::from(byte_index)] }.load(Ordering::Acquire) & (1 << bit) != 0
        //Converts a NonZeroUsize into an usize, unsize::from,NonZeroUsize已知不等于0的整数
        //pub fn load(&self, order: Ordering) -> u64 从原子整数加载值。load使用一个Ordering参数，该参数描述此操作的内存顺序。
    }

    #[inline]
    pub fn set_reserved(&self, index: u8, reserved: bool) {
        let byte_index = index / 64;
        let bit = index % 64;

        unsafe { &self.reservations[usize::from(byte_index)] }.fetch_or(u64::from(reserved) << bit, Ordering::AcqRel);
        //用当前值按位“或”。对当前值和参数val进行按位“或”运算，并将新值设置为结果。返回前一个值。
    }
    #[inline]
    pub fn is_reserved_mut(&mut self, index: u8) -> bool {//判断
        let byte_index = index / 64;
        let bit = index % 64;

        *unsafe { &mut self.reservations[usize::from(byte_index)] }.get_mut() & (1 << bit) != 0
        //pub fn get_mut(&mut self) -> &mut u64 返回对基础整数的可变引用。
    }

    #[inline]
    pub fn set_reserved_mut(&mut self, index: u8, reserved: bool) {
        let byte_index = index / 64;
        let bit = index % 64;

        *unsafe { &mut self.reservations[usize::from(byte_index)] }.get_mut() |= u64::from(reserved) << bit;
        //返回对基础整数的可变引用。安全的，因为是原子的
    }
}

static mut INIT_BSP_IDT: Idt = Idt::new();//声明了一个Idt变量

// TODO: VecMap?
pub static IDTS: RwLock<Option<BTreeMap<usize, &'static mut Idt>>> = RwLock::new(None);

#[inline]
pub fn is_reserved(cpu_id: usize, index: u8) -> bool {
    let byte_index = index / 64;
    let bit = index % 64;

    unsafe { &IDTS.read().as_ref().unwrap().get(&cpu_id).unwrap().reservations[usize::from(byte_index)] }.load(Ordering::Acquire) & (1 << bit) != 0
}

#[inline]
pub fn set_reserved(cpu_id: usize, index: u8, reserved: bool) {
    let byte_index = index / 64;
    let bit = index % 64;

    unsafe { &IDTS.read().as_ref().unwrap().get(&cpu_id).unwrap().reservations[usize::from(byte_index)] }.fetch_or(u64::from(reserved) << bit, Ordering::AcqRel);
}

pub fn allocate_interrupt() -> Option<NonZeroU8> {
    let cpu_id = crate::cpu_id();
    for number in 50..=254 {
        if ! is_reserved(cpu_id, number) {
            set_reserved(cpu_id, number, true);
            return Some(unsafe { NonZeroU8::new_unchecked(number) });
        }
    }
    None
}

pub fn available_irqs_iter(cpu_id: usize) -> impl Iterator<Item = u8> + 'static {
    (32..=254).filter(move |&index| !is_reserved(cpu_id, index))
}

macro_rules! use_irq(
    ( $idt: expr, $number:literal, $func:ident ) => {{
        $idt[$number].set_func($func);
    }}
);

macro_rules! use_default_irqs(
    ($idt:expr) => {{
        use crate::interrupt::irq::*;
        default_irqs!($idt, use_irq);
    }}
);

pub unsafe fn init() {//start.rs引用的函数
    dtables::lidt(&INIT_IDTR);
}

const fn new_idt_reservations() -> [AtomicU64; 4] {//原子数组的初始化
    [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)]
}

/// Initialize the IDT for a
pub unsafe fn init_paging_post_heap(is_bsp: bool, cpu_id: usize) {
    let mut idts_guard = IDTS.write();
    let idts_btree = idts_guard.get_or_insert_with(|| BTreeMap::new());

    if is_bsp {
        idts_btree.insert(cpu_id, &mut INIT_BSP_IDT);
    } else {
        let idt = idts_btree.entry(cpu_id).or_insert_with(|| Box::leak(Box::new(Idt::new())));
        init_generic(is_bsp, idt);
    }
}

/// Initializes a fully functional IDT for use before it be moved into the map. This is ONLY called
/// on the BSP, since the kernel heap is ready for the APs.
pub unsafe fn init_paging_bsp() {
    init_generic(true, &mut INIT_BSP_IDT);
}

/// Initializes an IDT for any type of processor.
pub unsafe fn init_generic(is_bsp: bool, idt: &mut Idt) {
    let (current_idt, current_reservations) = (&mut idt.entries, &mut idt.reservations);

    IDTR.limit = (current_idt.len() * mem::size_of::<IdtEntry>() - 1) as u16;
    IDTR.base = current_idt.as_ptr() as *const X86IdtEntry;

    // Set up exceptions
    current_idt[0].set_func(exception::divide_by_zero);
    current_idt[1].set_func(exception::debug);
    current_idt[2].set_func(exception::non_maskable);
    current_idt[3].set_func(exception::breakpoint);
    current_idt[3].set_flags(IdtFlags::PRESENT | IdtFlags::RING_3 | IdtFlags::INTERRUPT);
    current_idt[4].set_func(exception::overflow);
    current_idt[5].set_func(exception::bound_range);
    current_idt[6].set_func(exception::invalid_opcode);
    current_idt[7].set_func(exception::device_not_available);
    current_idt[8].set_func(exception::double_fault);
    // 9 no longer available
    current_idt[10].set_func(exception::invalid_tss);
    current_idt[11].set_func(exception::segment_not_present);
    current_idt[12].set_func(exception::stack_segment);
    current_idt[13].set_func(exception::protection);
    current_idt[14].set_func(exception::page);
    // 15 reserved
    current_idt[16].set_func(exception::fpu_fault);
    current_idt[17].set_func(exception::alignment_check);
    current_idt[18].set_func(exception::machine_check);
    current_idt[19].set_func(exception::simd);
    current_idt[20].set_func(exception::virtualization);
    // 21 through 29 reserved
    current_idt[30].set_func(exception::security);
    // 31 reserved

    // reserve bits 31:0, i.e. the first 32 interrupts, which are reserved for exceptions
    *current_reservations[0].get_mut() |= 0x0000_0000_FFFF_FFFF;

    if is_bsp {
        // Set up IRQs
        current_idt[32].set_func(irq::pit_stack);
        current_idt[33].set_func(irq::keyboard);
        current_idt[34].set_func(irq::cascade);
        current_idt[35].set_func(irq::com2);
        current_idt[36].set_func(irq::com1);
        current_idt[37].set_func(irq::lpt2);
        current_idt[38].set_func(irq::floppy);
        current_idt[39].set_func(irq::lpt1);
        current_idt[40].set_func(irq::rtc);
        current_idt[41].set_func(irq::pci1);
        current_idt[42].set_func(irq::pci2);
        current_idt[43].set_func(irq::pci3);
        current_idt[44].set_func(irq::mouse);
        current_idt[45].set_func(irq::fpu);
        current_idt[46].set_func(irq::ata1);
        current_idt[47].set_func(irq::ata2);
        current_idt[48].set_func(irq::lapic_timer);
        current_idt[49].set_func(irq::lapic_error);


        // reserve bits 49:32, which are for the standard IRQs, and for the local apic timer and error.
        *current_reservations[0].get_mut() |= 0x0003_FFFF_0000_0000;
    } else {
        // TODO: use_default_irqs! but also the legacy IRQs that are only needed on one CPU
    }

    use_default_irqs!(current_idt);

    // Set IPI handlers
    current_idt[IpiKind::Wakeup as usize].set_func(ipi::wakeup);
    current_idt[IpiKind::Switch as usize].set_func(ipi::switch);
    current_idt[IpiKind::Tlb as usize].set_func(ipi::tlb);
    current_idt[IpiKind::Pit as usize].set_func(ipi::pit);
    idt.set_reserved_mut(IpiKind::Wakeup as u8, true);
    idt.set_reserved_mut(IpiKind::Switch as u8, true);
    idt.set_reserved_mut(IpiKind::Tlb as u8, true);
    idt.set_reserved_mut(IpiKind::Pit as u8, true);
    let current_idt = &mut idt.entries;

    // Set syscall function
    current_idt[0x80].set_func(syscall::syscall);
    current_idt[0x80].set_flags(IdtFlags::PRESENT | IdtFlags::RING_3 | IdtFlags::INTERRUPT);
    idt.set_reserved_mut(0x80, true);

    dtables::lidt(&IDTR);
}

bitflags! {//属性位
    pub struct IdtFlags: u8 {
        const PRESENT = 1 << 7;
        const RING_0 = 0 << 5;
        const RING_1 = 1 << 5;
        const RING_2 = 2 << 5;
        const RING_3 = 3 << 5;
        const SS = 1 << 4;
        const INTERRUPT = 0xE;
        const TRAP = 0xF;
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct IdtEntry {//128位
    offsetl: u16,//跳转函数地址低16位
    selector: u16,//段选择子
    zero: u8,//填充0x00
    attribute: u8,//属性
    offsetm: u16,//中16位
    //之上为32位机器的中断表
    offseth: u32,//高32位
    zero2: u32//填充0x00
}

impl IdtEntry {
    pub const fn new() -> IdtEntry {
        IdtEntry {
            offsetl: 0,
            selector: 0,
            zero: 0,
            attribute: 0,
            offsetm: 0,
            offseth: 0,
            zero2: 0
        }
    }

    pub fn set_flags(&mut self, flags: IdtFlags) {//设置属性
        self.attribute = flags.bits;
    }

    pub fn set_offset(&mut self, selector: u16, base: usize) {//设置函数地址
        self.selector = selector;
        self.offsetl = base as u16;
        self.offsetm = (base >> 16) as u16;
        self.offseth = (base >> 32) as u32;
    }

    // A function to set the offset more easily
    pub fn set_func(&mut self, func: unsafe extern fn()) {
        self.set_flags(IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT);
        self.set_offset(8, func as usize);
    }
}
