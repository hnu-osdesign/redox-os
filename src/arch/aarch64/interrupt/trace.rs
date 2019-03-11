/// Get a stack trace
//TODO: Check for stack being mapped before dereferencing
#[inline(never)]
pub unsafe fn stack_trace() {
}

/// Get a symbol
//TODO: Do not create Elf object for every symbol lookup
#[inline(never)]
pub unsafe fn symbol_trace(addr: usize) {
}
