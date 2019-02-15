bitflags! {
    pub struct ExceptionClasses: u32 {
        const   SVC_INSN_IN_AARCH64_STATE = 0b10101 << 26;
        const   DATA_ABORT_FROM_LOWER_EL  = 0b100100 << 26;
        const   BKPT_INSN_IN_AARCH64_STATE = 0b111100 << 26;
    }
}

#[naked]
#[no_mangle]
pub unsafe extern fn report_exception() {
    let esr: usize;
    asm!("mrs   $0, esr_el1" : "=r"(esr) : : : "volatile");

    let exception_class = ExceptionClasses::from_bits_truncate(esr as u32);
    if exception_class.contains(ExceptionClasses::SVC_INSN_IN_AARCH64_STATE) {
        panic!("FATAL: SVC Instruction encountered in AArch64 state!");
    }

    panic!("FATAL: Unregistered exception encountered, ESR_EL1 = 0x{:x}", esr);
}
