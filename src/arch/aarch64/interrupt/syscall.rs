pub unsafe extern fn syscall() {
}

#[allow(dead_code)]
#[repr(packed)]
pub struct SyscallStack {
    pub rflags: usize,
}

#[naked]
pub unsafe extern fn clone_ret() {
}
