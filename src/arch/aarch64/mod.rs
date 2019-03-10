/// Constants like memory locations
pub mod consts;

#[macro_use]
pub mod macros;

/// Debugging support
pub mod debug;

/// Devices
pub mod device;

/// Interrupt instructions
pub mod interrupt;

/// Inter-processor interrupts
pub mod ipi;

/// Paging
pub mod paging;

/// Initialization and start function
pub mod start;

/// Stop function
pub mod stop;
