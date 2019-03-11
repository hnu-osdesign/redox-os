use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

//resets to 0 in context::switch()
pub static PIT_TICKS: AtomicUsize = ATOMIC_USIZE_INIT;

unsafe fn trigger(irq: u8) {
}

pub unsafe fn acknowledge(irq: usize) {
}
