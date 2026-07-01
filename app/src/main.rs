#![no_std]
#![no_main]
use core::panic::PanicInfo;
use core::arch::asm;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let msg = b"Hello from Ring 3!\n";
    unsafe {
        asm!(
            "syscall",
            in("rax") 1, // write
            in("rdi") 1, // stdout
            in("rsi") msg.as_ptr(),
            in("rdx") msg.len(),
            out("rcx") _,
            out("r11") _,
        );
        
        asm!(
            "syscall",
            in("rax") 60, // exit
            in("rdi") 0,
            options(noreturn)
        );
    }
}
