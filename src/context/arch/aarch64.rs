use core::mem;
use core::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT};

/// This must be used by the kernel to ensure that context switches are done atomically
/// Compare and exchange this to true when beginning a context switch on any CPU
/// The `Context::switch_to` function will set it back to false, allowing other CPU's to switch
/// This must be done, as no locks can be held on the stack during switch
pub static CONTEXT_SWITCH_LOCK: AtomicBool = ATOMIC_BOOL_INIT;

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

    pub fn get_page_table(&self) -> usize {
        self.cr3
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
    }

    pub unsafe fn pop_stack(&mut self) -> usize {
        0
    }

    /// Switch to the next context by restoring its stack and registers
    #[cold]
    #[inline(never)]
    #[naked]
    pub unsafe fn switch_to(&mut self, next: &mut Context) {
    }
}

#[allow(dead_code)]
#[repr(packed)]
pub struct SignalHandlerStack {
    handler: extern fn(usize),
    sig: usize,
    rip: usize,
}

#[naked]
unsafe extern fn signal_handler_wrapper() {
    // Push scratch registers

    // Get reference to stack variables

    // Call inner rust function

    // Pop scratch registers, error code, and return
}
