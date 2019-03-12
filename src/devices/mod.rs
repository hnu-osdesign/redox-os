#[cfg(target_arch = "x86_64")]
pub mod uart_16550;
#[cfg(target_arch = "aarch64")]
pub mod uart_pl011;
