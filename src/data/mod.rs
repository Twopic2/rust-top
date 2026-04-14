pub mod cpu;
pub mod mem;
pub mod os;
pub mod clock;

pub mod temp;
pub mod network;
pub mod disk;
#[cfg(target_os = "macos")]
pub mod darwin;
