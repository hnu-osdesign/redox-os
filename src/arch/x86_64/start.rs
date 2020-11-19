///  这个函数是用于内核设置中断处理程序
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are defined in other files inside of the `arch` module

use core::slice;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::allocator;
#[cfg(feature = "acpi")]
use crate::acpi;
#[cfg(feature = "graphical_debug")]
use crate::arch::x86_64::graphical_debug;
use crate::arch::x86_64::pti;
use crate::arch::x86_64::flags::*;
use crate::device;
use crate::gdt;
use crate::idt;
use crate::interrupt;
use crate::log::{self, info};
use crate::memory;
use crate::paging;

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

pub static KERNEL_BASE: AtomicUsize = AtomicUsize::new(0);
pub static KERNEL_SIZE: AtomicUsize = AtomicUsize::new(0);
pub static CPU_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static AP_READY: AtomicBool = AtomicBool::new(false);
static BSP_READY: AtomicBool = AtomicBool::new(false);

#[repr(packed)]
pub struct KernelArgs {
    kernel_base: u64,
    kernel_size: u64,
    stack_base: u64,
    stack_size: u64,
    env_base: u64,
    env_size: u64,

    /// The base 64-bit pointer to an array of saved RSDPs. It's up to the kernel (and possibly
    /// userspace), to decide which RSDP to use. The buffer will be a linked list containing a
    /// 32-bit relative (to this field) next, and the actual struct afterwards.
    ///
    /// This field can be NULL, and if so, the system has not booted with UEFI or in some other way
    /// retrieved the RSDPs. The kernel or a userspace driver will thus try searching the BIOS
    /// memory instead. On UEFI systems, searching is not guaranteed to actually work though.
    acpi_rsdps_base: u64,
    /// The size of the RSDPs region.
    acpi_rsdps_size: u64,
}

/// rust入口点，所有都需要初始化。
#[no_mangle]
pub unsafe extern fn kstart(args_ptr: *const KernelArgs) -> ! {
    let env = {
        ///env=slice::from_raw_parts(env_base as *const u8, env_size);
        let args = &*args_ptr;

        let kernel_base = args.kernel_base as usize;
        let kernel_size = args.kernel_size as usize;
        let stack_base = args.stack_base as usize;
        let stack_size = args.stack_size as usize;
        let env_base = args.env_base as usize;
        let env_size = args.env_size as usize;
        let acpi_rsdps_base = args.acpi_rsdps_base;
        let acpi_rsdps_size = args.acpi_rsdps_size;

        // BSS should already be zero ，BSS段
        {
            assert_eq!(BSS_TEST_ZERO, 0);
            assert_eq!(DATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
        }

        KERNEL_BASE.store(kernel_base, Ordering::SeqCst);
        KERNEL_SIZE.store(kernel_size, Ordering::SeqCst);

        // Initialize logger 初始化日志
        log::init_logger(|r| {
            use core::fmt::Write;
            let _ = write!(
                crate::arch::x86_64::debug::Writer::new(),
                "{}:{} -- {}\n",
                r.target(),
                r.level(),
                r.args()
            );
        });

        info!("Redox OS starting...");
        info!("Kernel: {:X}:{:X}", kernel_base, kernel_base + kernel_size);
        info!("Stack: {:X}:{:X}", stack_base, stack_base + stack_size);
        info!("Env: {:X}:{:X}", env_base, env_base + env_size);
        info!("RSDPs: {:X}:{:X}", acpi_rsdps_base, acpi_rsdps_base + acpi_rsdps_size);
        //Remove unnecessary kernel args. 移除不必要的内核参数。
        let ext_mem_ranges = if args.acpi_rsdps_base != 0 && args.acpi_rsdps_size > 0 {
            Some([(acpi_rsdps_base as usize, acpi_rsdps_size as usize)])
        } else {
            None
        };

        // Set up GDT before paging 在分页之前设置GDT(全局段表)
        gdt::init();

        // Set up IDT before paging 在分页之前设置IDT（中断描述符表）
        idt::init();

        // Initialize memory management 初始化内存模块
        memory::init(0, kernel_base + ((kernel_size + 4095)/4096) * 4096);

        // Initialize paging 初始化分页
        let (mut active_table, tcb_offset) = paging::init(0, kernel_base, kernel_base + kernel_size, stack_base, stack_base + stack_size, ext_mem_ranges.as_ref().map(|arr| &arr[..]).unwrap_or(&[]));

        // Set up GDT after paging with TLS 设置GDT在使用TLS(线程局部存储)的分页之后。
        gdt::init_paging(tcb_offset, stack_base + stack_size);

        // Set up IDT 设置IDT（中断表）
        idt::init_paging_bsp();

        // Set up syscall instruction 设置系统调用指令
        interrupt::syscall::init();

        // Test tdata and tbss
        {
            assert_eq!(TBSS_TEST_ZERO, 0);
            TBSS_TEST_ZERO += 1;
            assert_eq!(TBSS_TEST_ZERO, 1);
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
            TDATA_TEST_NONZERO -= 1;
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFE);
        }

        // Reset AP variables 设置AP变量（用于副CPU）
        CPU_COUNT.store(1, Ordering::SeqCst);
        AP_READY.store(false, Ordering::SeqCst);
        BSP_READY.store(false, Ordering::SeqCst);

        // Setup kernel heap 设置内核堆
        allocator::init(&mut active_table);

        idt::init_paging_post_heap(true, 0);

        // Activate memory logging 激活内存日志。
        log::init();

        // Use graphical debug 使用图形化debug
        #[cfg(feature="graphical_debug")]
        graphical_debug::init(&mut active_table);

        #[cfg(feature = "system76_ec_debug")]
        device::system76_ec::init();

        // Initialize devices 初始化设备
        device::init(&mut active_table);

        // Read ACPI tables, starts APs 初始化acpi（高级配置和电源管理接口）和设备
        #[cfg(feature = "acpi")]
        {
            acpi::init(&mut active_table, if acpi_rsdps_base != 0 && acpi_rsdps_size > 0 { Some((acpi_rsdps_base, acpi_rsdps_size)) } else { None });
            device::init_after_acpi(&mut active_table);
        }

        // Initialize all of the non-core devices not otherwise needed to complete initialization
        device::init_noncore();

        // Initialize memory functions after core has loaded
        memory::init_noncore();

        // Stop graphical debug
        #[cfg(feature="graphical_debug")]
        graphical_debug::fini(&mut active_table);

        BSP_READY.store(true, Ordering::SeqCst);

        slice::from_raw_parts(env_base as *const u8, env_size)
    };

    crate::kmain(CPU_COUNT.load(Ordering::SeqCst), env);
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
    let cpu_id = {
        let args = &*args_ptr;
        let cpu_id = args.cpu_id as usize;
        let bsp_table = args.page_table as usize;
        let stack_start = args.stack_start as usize;
        let stack_end = args.stack_end as usize;

        assert_eq!(BSS_TEST_ZERO, 0);
        assert_eq!(DATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);

        // Set up GDT before paging
        gdt::init();

        // Set up IDT before paging
        idt::init();

        // Initialize paging
        let tcb_offset = paging::init_ap(cpu_id, bsp_table, stack_start, stack_end);

        // Set up GDT with TLS
        gdt::init_paging(tcb_offset, stack_end);

        // Set up IDT for AP
        idt::init_paging_post_heap(false, cpu_id);

        // Set up syscall instruction
        interrupt::syscall::init();

        // Test tdata and tbss
        {
            assert_eq!(TBSS_TEST_ZERO, 0);
            TBSS_TEST_ZERO += 1;
            assert_eq!(TBSS_TEST_ZERO, 1);
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
            TDATA_TEST_NONZERO -= 1;
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFE);
        }

        // Initialize devices (for AP)
        device::init_ap();

        AP_READY.store(true, Ordering::SeqCst);

        cpu_id
    };

    while ! BSP_READY.load(Ordering::SeqCst) {
        interrupt::pause();
    }

    crate::kmain_ap(cpu_id);
}

#[naked]
pub unsafe fn usermode(ip: usize, sp: usize, arg: usize, singlestep: bool) -> ! {
    let mut flags = FLAG_INTERRUPTS;
    if singlestep {
        flags |= FLAG_SINGLESTEP;
    }

    asm!("push r10
          push r11
          push r12
          push r13
          push r14
          push r15",
         in("r10") (gdt::GDT_USER_DATA << 3 | 3), // Data segment
         in("r11") sp, // Stack pointer
         in("r12") flags, // Flags
         in("r13") (gdt::GDT_USER_CODE << 3 | 3), // Code segment
         in("r14") ip, // IP
         in("r15") arg, // Argument
    );

    // Unmap kernel
    pti::unmap();

    // Go to usermode
    asm!("mov ds, r14d
         mov es, r14d
         mov fs, r15d
         mov gs, r14d
         xor rax, rax
         xor rbx, rbx
         xor rcx, rcx
         xor rdx, rdx
         xor rsi, rsi
         xor rdi, rdi
         xor rbp, rbp
         xor r8, r8
         xor r9, r9
         xor r10, r10
         xor r11, r11
         xor r12, r12
         xor r13, r13
         xor r14, r14
         xor r15, r15
         fninit
         pop rdi
         iretq",
         in("r14") (gdt::GDT_USER_DATA << 3 | 3), // Data segment
         in("r15") (gdt::GDT_USER_TLS << 3 | 3), // TLS segment
         options(noreturn),
    );
}
