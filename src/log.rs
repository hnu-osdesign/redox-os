use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

pub static LOG: Mutex<Option<Log>> = Mutex::new(None);//定义全局变量LOG为Mutex类型，参数Option，如果有值就定义为Log类型的。

pub fn init() {
    *LOG.lock() = Some(Log::new(1024 * 1024));//返回一个option，使值为Some，绑定在LOG的data上。
}

pub struct Log {//log 结构体，包含数据和数据的大小。
    data: VecDeque<u8>,
    size: usize,
}

impl Log {
    pub fn new(size: usize) -> Log {//方法new()，创建一个Log
        Log {
            data: VecDeque::with_capacity(size),//容量为size
            size
        }
    }

    pub fn read(&self) -> (&[u8], &[u8]) {
        self.data.as_slices()
    }

    pub fn write(&mut self, buf: &[u8]) {
        for &b in buf {
            while self.data.len() + 1 >= self.size {
                self.data.pop_front();
            }
            self.data.push_back(b);
        }
    }
}

struct RedoxLogger {
    log_func: fn(&log::Record),
    pub initialized: AtomicBool,
}

impl ::log::Log for RedoxLogger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        false
    }
    fn log(&self, record: &log::Record<'_>) {
        (self.log_func)(&record)
    }
    fn flush(&self) {}
}

pub fn init_logger(func: fn(&log::Record)) {
    unsafe {
        match LOGGER.initialized.load(Ordering::SeqCst) {
            false => {
                ::log::set_max_level(::log::LevelFilter::Info);
                    LOGGER.log_func = func;
                    match ::log::set_logger(&LOGGER) {
                        Ok(_) => ::log::info!("Logger initialized."),
                        Err(e) => println!("Logger setup failed! error: {}", e),
                    }
                LOGGER.initialized.store(true, Ordering::SeqCst);
            },
            true => ::log::info!("Tried to reinitialize the logger, which is not possible. Ignoring."),
        }
    }
}

static mut LOGGER: RedoxLogger = RedoxLogger {
    log_func: |_| {},
    initialized: AtomicBool::new(false),
};

pub use log::{debug, error, info, set_max_level, warn};
