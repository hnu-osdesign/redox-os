use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};
use syscall::data::FloatRegisters;

/// This must be used by the kernel to ensure that context switches are done atomically
/// Compare and exchange this to true when beginning a context switch on any CPU
/// The `Context::switch_to` function will set it back to false, allowing other CPU's to switch
/// This must be done, as no locks can be held on the stack during switch
pub static CONTEXT_SWITCH_LOCK: AtomicBool = AtomicBool::new(false);

const ST_RESERVED: u128 = 0xFFFF_FFFF_FFFF_0000_0000_0000_0000_0000;

#[derive(Clone, Debug)]
pub struct Context {
    /// FX valid?
    loadable: bool,
    /// FX location
    fx: usize,
    /// Page table pointer
    cr3: usize,
    /// RFLAGS register
    rflags: usize,
    /// RBX register
    rbx: usize,
    /// R12 register
    r12: usize,
    /// R13 register
    r13: usize,
    /// R14 register
    r14: usize,
    /// R15 register
    r15: usize,
    /// Base pointer
    rbp: usize,
    /// Stack pointer
    rsp: usize
}

impl Context {
    pub fn new() -> Context {
        Context {
            loadable: false,
            fx: 0,
            cr3: 0,
            rflags: 0,
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rbp: 0,
            rsp: 0
        }
    }

    pub fn get_page_table(&mut self) -> usize {
        self.cr3
    }

    pub fn get_fx_regs(&self) -> Option<FloatRegisters> {
        if !self.loadable {
            return None;
        }
        let mut regs = unsafe { *(self.fx as *const FloatRegisters) };
        regs._reserved = 0;
        let mut new_st = regs.st_space;
        for st in &mut new_st {
            // Only allow access to the 80 lowest bits
            *st &= !ST_RESERVED;
        }
        regs.st_space = new_st;
        Some(regs)
    }

    pub fn set_fx_regs(&mut self, mut new: FloatRegisters) -> bool {
        if !self.loadable {
            return false;
        }

        {
            let old = unsafe { &*(self.fx as *const FloatRegisters) };
            new._reserved = old._reserved;
            let old_st = new.st_space;
            let mut new_st = new.st_space;
            for (new_st, old_st) in new_st.iter_mut().zip(&old_st) {
                *new_st &= !ST_RESERVED;
                *new_st |= old_st & ST_RESERVED;
            }
            new.st_space = new_st;

            // Make sure we don't use `old` from now on
        }

        unsafe {
            *(self.fx as *mut FloatRegisters) = new;
        }
        true
    }

    pub fn set_fx(&mut self, address: usize) {
        self.fx = address;
    }

    pub fn set_page_table(&mut self, address: usize) {
        self.cr3 = address;
    }

    pub fn set_stack(&mut self, address: usize) {
        self.rsp = address;
    }

    pub unsafe fn signal_stack(&mut self, handler: extern fn(usize), sig: u8) {
        self.push_stack(sig as usize);
        self.push_stack(handler as usize);
        self.push_stack(signal_handler_wrapper as usize);
    }

    pub unsafe fn push_stack(&mut self, value: usize) {
        self.rsp -= mem::size_of::<usize>();
        *(self.rsp as *mut usize) = value;
    }

    pub unsafe fn pop_stack(&mut self) -> usize {
        let value = *(self.rsp as *const usize);
        self.rsp += mem::size_of::<usize>();
        value
    }

    /// Switch to the next context by restoring its stack and registers
    /// Check disassembly!
    #[cold]
    #[inline(never)]
    #[naked]
    pub unsafe fn switch_to(&mut self, next: &mut Context) {
        asm!("fxsave64 [{}]", in(reg) (self.fx));
        self.loadable = true;
        if next.loadable {
            asm!("fxrstor64 [{}]", in(reg) (next.fx));
        }else{
            asm!("fninit");
        }

        asm!("mov {}, cr3", out(reg) (self.cr3));
        if next.cr3 != self.cr3 {
            asm!("mov cr3, {}", in(reg) (next.cr3));
        }

        asm!("pushfq ; pop {}", out(reg) (self.rflags));
        asm!("push {} ; popfq", in(reg) (next.rflags));

        asm!("mov {}, rbx", out(reg) (self.rbx));
        asm!("mov rbx, {}", in(reg) (next.rbx));

        asm!("mov {}, r12", out(reg) (self.r12));
        asm!("mov r12, {}", in(reg) (next.r12));

        asm!("mov {}, r13", out(reg) (self.r13));
        asm!("mov r13, {}", in(reg) (next.r13));

        asm!("mov {}, r14", out(reg) (self.r14));
        asm!("mov r14, {}", in(reg) (next.r14));

        asm!("mov {}, r15", out(reg) (self.r15));
        asm!("mov r15, {}", in(reg) (next.r15));

        asm!("mov {}, rsp", out(reg) (self.rsp));
        asm!("mov rsp, {}", in(reg) (next.rsp));

        asm!("mov {}, rbp", out(reg) (self.rbp));
        asm!("mov rbp, {}", in(reg) (next.rbp));

        // Unset global lock after loading registers but before switch
        CONTEXT_SWITCH_LOCK.store(false, Ordering::SeqCst);
    }
}

#[allow(dead_code)]
#[repr(packed)]
pub struct SignalHandlerStack {
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rsi: usize,
    rdi: usize,
    rdx: usize,
    rcx: usize,
    rax: usize,
    handler: extern fn(usize),
    sig: usize,
    rip: usize,
}

#[naked]
unsafe extern fn signal_handler_wrapper() {
    #[inline(never)]
    unsafe fn inner(stack: &SignalHandlerStack) {
        (stack.handler)(stack.sig);
    }

    // Push scratch registers
    asm!("push rax
        push rcx
        push rdx
        push rdi
        push rsi
        push r8
        push r9
        push r10
        push r11");

    // Get reference to stack variables
    let rsp: usize;
    asm!("mov {}, rsp", out(reg) rsp);

    // Call inner rust function
    inner(&*(rsp as *const SignalHandlerStack));

    // Pop scratch registers, error code, and return
    asm!("pop r11
        pop r10
        pop r9
        pop r8
        pop rsi
        pop rdi
        pop rdx
        pop rcx
        pop rax
        add rsp, 16");
}
