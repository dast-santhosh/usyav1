#![no_std]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupts;
pub mod pci;
pub mod serial;

pub fn init() {
    serial::SERIAL1.lock().init();
    gdt::init();
    interrupts::init();
    unsafe { interrupts::PICS.lock().initialize() };
}
