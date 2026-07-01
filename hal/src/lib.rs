#![no_std]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupts;

pub fn init() {
    gdt::init();
    interrupts::init();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
