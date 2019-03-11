#[no_mangle]
pub unsafe extern fn kstop() -> ! {
    loop {}
}
