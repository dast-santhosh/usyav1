#![no_std]

pub mod syscalls;
pub mod process;

pub fn init() {
    syscalls::init();
}
