use paging::ActivePageTable;

pub mod cpu;
pub mod gic;
pub mod generic_timer;
pub mod serial;
pub mod rtc;

pub unsafe fn init(_active_table: &mut ActivePageTable) {
    gic::init();
    generic_timer::init();
}

pub unsafe fn init_noncore() {
    serial::init();
    rtc::init();
}

pub unsafe fn init_ap() {
}
