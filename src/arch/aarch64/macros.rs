/// Print to console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::arch::debug::Writer::new(), $($arg)*);
    });
}

/// Print with new line to console
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[allow(dead_code)]
#[repr(packed)]
pub struct ScratchRegisters {
}

impl ScratchRegisters {
    pub fn dump(&self) {
    }
}

macro_rules! scratch_push {
    () => (asm!(
        "nop"
        : : : : "volatile"
    ));
}

macro_rules! scratch_pop {
    () => (asm!(
        "nop"
        : : : : "volatile"
    ));
}

#[allow(dead_code)]
#[repr(packed)]
pub struct PreservedRegisters {
}

impl PreservedRegisters {
    pub fn dump(&self) {
    }
}

macro_rules! preserved_push {
    () => (asm!(
        "nop"
        : : : : "volatile"
    ));
}

macro_rules! preserved_pop {
    () => (asm!(
        "nop"
        : : : : "volatile"
    ));
}

macro_rules! fs_push {
    () => (asm!(
        "nop"
        : : : : "volatile"
    ));
}

macro_rules! fs_pop {
    () => (asm!(
        "nop"
        : : : : "volatile"
    ));
}

#[allow(dead_code)]
#[repr(packed)]
pub struct IretRegisters {
}

impl IretRegisters {
    pub fn dump(&self) {
    }
}

macro_rules! iret {
    () => (asm!(
        "nop"
        : : : : "volatile"
    ));
}

/// Create an interrupt function that can safely run rust code
#[macro_export]
macro_rules! interrupt {
    ($name:ident, $func:block) => {
        #[naked]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner() {
                $func
            }

            // Push scratch registers
            scratch_push!();
            fs_push!();

            // Call inner rust function
            inner();

            // Pop scratch registers and return
            fs_pop!();
            scratch_pop!();
            iret!();
        }
    };
}

#[allow(dead_code)]
#[repr(packed)]
pub struct InterruptStack {
}

impl InterruptStack {
    pub fn dump(&self) {
    }
}

#[macro_export]
macro_rules! interrupt_stack {
    ($name:ident, $stack: ident, $func:block) => {
        #[naked]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner($stack: &mut $crate::arch::aarch64::macros::InterruptStack) {
                $func
            }

            // Push scratch registers
            scratch_push!();
            fs_push!();

            // Get reference to stack variables
            let rsp: usize;
            asm!("" : "={rsp}"(sp) : : : "volatile");

            // Call inner rust function
            inner(&mut *(rsp as *mut $crate::arch::aarch64::macros::InterruptStack));

            // Pop scratch registers and return
            fs_pop!();
            scratch_pop!();
            iret!();
        }
    };
}

#[allow(dead_code)]
#[repr(packed)]
pub struct InterruptErrorStack {
}

impl InterruptErrorStack {
    pub fn dump(&self) {
    }
}

#[macro_export]
macro_rules! interrupt_error {
    ($name:ident, $stack:ident, $func:block) => {
        #[naked]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner($stack: &$crate::arch::aarch64::macros::InterruptErrorStack) {
                $func
            }

            // Push scratch registers
            scratch_push!();
            fs_push!();

            // Get reference to stack variables
            let rsp: usize;
            asm!("" : "={rsp}"(sp) : : : "volatile");

            // Call inner rust function
            inner(&*(rsp as *const $crate::arch::aarch64::macros::InterruptErrorStack));

            // Pop scratch registers, error code, and return
            fs_pop!();
            scratch_pop!();
            iret!();
        }
    };
}

#[allow(dead_code)]
#[repr(packed)]
pub struct InterruptStackP {
}

impl InterruptStackP {
    pub fn dump(&self) {
    }
}

#[macro_export]
macro_rules! interrupt_stack_p {
    ($name:ident, $stack: ident, $func:block) => {
        #[naked]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner($stack: &mut $crate::arch::aarch64::macros::InterruptStackP) {
                $func
            }

            // Push scratch registers
            scratch_push!();
            preserved_push!();
            fs_push!();

            // Get reference to stack variables
            let rsp: usize;
            asm!("" : "={rsp}"(sp) : : : "volatile");

            // Call inner rust function
            inner(&mut *(rsp as *mut $crate::arch::aarch64::macros::InterruptStackP));

            // Pop scratch registers and return
            fs_pop!();
            preserved_pop!();
            scratch_pop!();
            iret!();
        }
    };
}

#[allow(dead_code)]
#[repr(packed)]
pub struct InterruptErrorStackP {
}

impl InterruptErrorStackP {
    pub fn dump(&self) {
    }
}

#[macro_export]
macro_rules! interrupt_error_p {
    ($name:ident, $stack:ident, $func:block) => {
        #[naked]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner($stack: &$crate::arch::aarch64::macros::InterruptErrorStackP) {
                $func
            }

            // Push scratch registers
            scratch_push!();
            preserved_push!();
            fs_push!();

            // Get reference to stack variables
            let rsp: usize;
            asm!("" : "={rsp}"(rsp) : : : "intel", "volatile");

            // Call inner rust function
            inner(&*(rsp as *const $crate::arch::aarch64::macros::InterruptErrorStackP));

            // Pop scratch registers, error code, and return
            fs_pop!();
            preserved_pop!();
            scratch_pop!();
            iret!();
        }
    };
}
