//! Interrupt instructions

pub mod ipi;
pub mod irq;
pub mod syscall;
pub mod trace;

pub use self::trace::stack_trace;

/// Clear interrupts
#[inline(always)]
pub unsafe fn disable() {
}

/// Set interrupts
#[inline(always)]
pub unsafe fn enable() {
}

/// Set interrupts and halt
/// This will atomically wait for the next interrupt
/// Performing enable followed by halt is not guaranteed to be atomic, use this instead!
#[inline(always)]
pub unsafe fn enable_and_halt() {
}

/// Set interrupts and nop
/// This will enable interrupts and allow the IF flag to be processed
/// Simply enabling interrupts does not gurantee that they will trigger, use this instead!
#[inline(always)]
pub unsafe fn enable_and_nop() {
}

/// Halt instruction
#[inline(always)]
pub unsafe fn halt() {
}

/// Pause instruction
/// Safe because it is similar to a NOP, and has no memory effects
#[inline(always)]
pub fn pause() {
}
