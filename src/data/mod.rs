pub mod cpu;
pub mod mem;
pub mod os;

pub mod temp;
pub mod network;
pub mod disk;
pub mod power;

#[cfg(target_os = "macos")]
pub mod darwin;
