#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use gpu::Compositor;
use x86_64::instructions::interrupts;
use input::keyboard::KEYBOARD_QUEUE;
use input::mouse::MOUSE;
use core::fmt::Write;
use hal::{serial_print, serial_println};

entry_point!(kernel_main);

// 8MB back buffer should be enough for up to 1920x1080x4
static mut BACK_BUFFER: [u8; 1920 * 1080 * 4] = [0; 1920 * 1080 * 4];

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("[USYA NANO-CORE] Kernel successfully booted. Ring 0 active.");

    /*
    // COMMENTED OUT TO ISOLATE KERNEL PANIC
    */
    // Initialize HAL (GDT, IDT, PIC)
    hal::init();
    input::init();

    /*

    // Initialize Memory Subsystem (PMM, Paging, Heap)
    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    
    let mut mapper = unsafe { kernel::memory::paging::init(phys_mem_offset) };
    let mut frame_allocator = kernel::memory::pmm::BitmapFrameAllocator::new();
    frame_allocator.init(&boot_info.memory_regions);
    kernel::memory::paging::identity_map_2gb(&mut mapper, &mut frame_allocator);
    kernel::memory::heap::init_heap(&mut mapper, &mut frame_allocator).expect("Heap init failed");
    kernel::memory::init_global_memory(mapper, frame_allocator);

    interrupts::enable();
    posix::init();

    let elf_data = include_bytes!("../../c_template/driver.elf");
    posix::process::start_process(elf_data, boot_info.physical_memory_offset.into_option().unwrap());
    */

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let mut compositor = Compositor::new(framebuffer, unsafe { &mut *(&raw mut BACK_BUFFER) });

        // 1. Clear background to pure white
        compositor.clear(255, 255, 255);
        
        // 2. Draw a simple red square to prove visuals work
        compositor.draw_rect(100, 100, 50, 50, 255, 0, 0);

        // 3. Flip buffers to screen
        compositor.swap_buffers();
        
        serial_println!("[USYA NANO-CORE] Framebuffer compositor initialized. Safe Halting.");
    }

    // Explicit infinite loop so the CPU halts safely without rebooting
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[KERNEL PANIC] {}", info);
    // Explicit infinite loop. No int3 or hlt since IDT might be uninitialized.
    loop {}
}
