#![no_std]

pub mod keyboard;
pub mod mouse;

pub fn init() {
    mouse::MOUSE.lock().init();
}
