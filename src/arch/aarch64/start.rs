/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are
/// defined in other files inside of the `arch` module

use core::slice;
use core::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

use memory::{Frame};
use paging::{ActivePageTable, PageTableType, Page, PAGE_SIZE, PhysicalAddress, VirtualAddress};
use paging::entry::{EntryFlags};

use allocator;
use device;
use init::device_tree;
use interrupt;
use memory;
use paging;

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0;
/// Test of non-zero values in data.
static DATA_TEST_NONZERO: usize = 0xFFFF_FFFF_FFFF_FFFF;
/// Test of zero values in thread BSS
#[thread_local]
static mut TBSS_TEST_ZERO: usize = 0;
/// Test of non-zero values in thread data.
#[thread_local]
static mut TDATA_TEST_NONZERO: usize = 0xFFFF_FFFF_FFFF_FFFF;

pub static KERNEL_BASE: AtomicUsize = ATOMIC_USIZE_INIT;
pub static KERNEL_SIZE: AtomicUsize = ATOMIC_USIZE_INIT;
pub static CPU_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
pub static AP_READY: AtomicBool = ATOMIC_BOOL_INIT;
static BSP_READY: AtomicBool = ATOMIC_BOOL_INIT;

#[repr(packed)]
pub struct KernelArgs {
    kernel_base: u64,
    kernel_size: u64,
    stack_base: u64,
    stack_size: u64,
    env_base: u64,
    env_size: u64,
    dtb_base: u64,
    dtb_size: u64,
}

/// The entry to Rust, all things must be initialized
#[no_mangle]
pub unsafe extern fn kstart(args_ptr: *const KernelArgs) -> ! {
    let env = {
        let args = &*args_ptr;

        let kernel_base = args.kernel_base as usize;
        let kernel_size = args.kernel_size as usize;
        let stack_base = args.stack_base as usize;
        let stack_size = args.stack_size as usize;
        let env_base = args.env_base as usize;
        let dtb_base = args.dtb_base as usize;
        let dtb_size = 0x200000;

        // BSS should already be zero
        {
            assert_eq!(BSS_TEST_ZERO, 0);
            assert_eq!(DATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
        }

        KERNEL_BASE.store(kernel_base, Ordering::SeqCst);
        KERNEL_SIZE.store(kernel_size, Ordering::SeqCst);

        device_tree::fill_memory_map(::KERNEL_DTB_OFFSET, dtb_size);
        let env_size = device_tree::fill_env_data(::KERNEL_DTB_OFFSET, dtb_size, env_base);

        // Initialize memory management
        memory::init(kernel_base, kernel_base + ((kernel_size + 4095)/4096) * 4096);

        // Initialize paging
        let (mut active_ktable, _tcb_offset) = paging::init(0, kernel_base, kernel_base + kernel_size,
                                                            stack_base, stack_base + stack_size,
                                                            dtb_base, dtb_base + dtb_size);

        // Test tdata and tbss
        {
            assert_eq!(TBSS_TEST_ZERO, 0);
            TBSS_TEST_ZERO += 1;
            assert_eq!(TBSS_TEST_ZERO, 1);
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
            TDATA_TEST_NONZERO -= 1;
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFE);
        }

        // Reset AP variables
        CPU_COUNT.store(1, Ordering::SeqCst);
        AP_READY.store(false, Ordering::SeqCst);
        BSP_READY.store(false, Ordering::SeqCst);

        // Setup kernel heap
        allocator::init(&mut active_ktable);

        // Initialize devices
        device::init(&mut active_ktable);

        // Initialize all of the non-core devices not otherwise needed to complete initialization
        device::init_noncore();

        println!("Kernel: {:X}:{:X}", kernel_base, kernel_base + kernel_size);
        println!("Stack: {:X}:{:X}", stack_base, stack_base + stack_size);
        println!("DTB: {:X}:{:X}", dtb_base, dtb_base + dtb_size);
        println!("Env: {:X}:{:X}", env_base, env_base + env_size);

        // Initialize memory functions after core has loaded
        memory::init_noncore();

        BSP_READY.store(true, Ordering::SeqCst);

        slice::from_raw_parts(env_base as *const u8, env_size - 1)
    };

    ::kmain(CPU_COUNT.load(Ordering::SeqCst), env);
}

#[repr(packed)]
pub struct KernelArgsAp {
    cpu_id: u64,
    page_table: u64,
    stack_start: u64,
    stack_end: u64,
}

/// Entry to rust for an AP
pub unsafe extern fn kstart_ap(args_ptr: *const KernelArgsAp) -> ! {
    loop{}
}

#[naked]
pub unsafe fn usermode(ip: usize, sp: usize, arg: usize) -> ! {
    let cpu_id: usize = 0;
    let uspace_tls_start = (::USER_TLS_OFFSET + ::USER_TLS_SIZE * cpu_id);
    let spsr: u32 = 0;

    asm!("msr   tpidr_el0, $0" : : "r"(uspace_tls_start) : : "volatile");
    asm!("msr   spsr_el1, $0" : : "r"(spsr) : : "volatile");
    asm!("msr   elr_el1, $0" : : "r"(ip) : : "volatile");
    asm!("msr   sp_el0, $0" : : "r"(sp) : : "volatile");

    asm!("mov   x0, $0" : : "r"(arg) : : "volatile");
    asm!("eret" : : : : "volatile");

    unreachable!();
}
