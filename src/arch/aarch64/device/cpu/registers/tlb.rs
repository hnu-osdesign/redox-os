//! Functions to flush the translation lookaside buffer (TLB).

pub unsafe fn flush(_addr: usize) {
    asm!("tlbi vmalle1is");
}

pub unsafe fn flush_all() {
    asm!("tlbi vmalle1is");
}
