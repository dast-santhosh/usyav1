#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use gpu::Compositor;
use x86_64::instructions::interrupts;
use input::keyboard::KEYBOARD_QUEUE;
use input::mouse::MOUSE;
use core::fmt::Write;

entry_point!(kernel_main);

// 8MB back buffer should be enough for up to 1920x1080x4
static mut BACK_BUFFER: [u8; 1920 * 1080 * 4] = [0; 1920 * 1080 * 4];

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialize HAL (GDT, IDT, PIC)
    hal::init();
    input::init();

    // Enable interrupts now that IDT and PIC are set up
    interrupts::enable();

    // Initialize Memory Subsystem (Paging, PMM)
    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let _mapper = unsafe { kernel::memory::init(phys_mem_offset) };
    let _frame_allocator = unsafe { kernel::memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let mut compositor = Compositor::new(framebuffer, unsafe { &mut BACK_BUFFER });

        let mut terminal_buffer: [char; 2048] = ['\0'; 2048];
        let mut terminal_len = 0;

        loop {
            // Check for new keyboard input
            while let Some(c) = KEYBOARD_QUEUE.lock().pop() {
                if c == '\x08' {
                    if terminal_len > 0 {
                        terminal_len -= 1;
                        terminal_buffer[terminal_len] = '\0';
                    }
                } else if terminal_len < 2048 {
                    terminal_buffer[terminal_len] = c;
                    terminal_len += 1;
                }
            }

            // Get mouse position
            let (mx, my) = {
                let mouse = MOUSE.lock();
                (mouse.x as usize, mouse.y as usize)
            };

            // 1. Clear background
            compositor.clear(40, 44, 52); // Dark grey background

            // 2. Draw terminal text
            let mut text_x = 10;
            let mut text_y = 10;
            for i in 0..terminal_len {
                let c = terminal_buffer[i];
                if c == '\n' {
                    text_y += 16;
                    text_x = 10;
                } else {
                    compositor.draw_char(text_x, text_y, c, 220, 220, 220); // Light text
                    text_x += 8;
                    if text_x + 8 > compositor.width() {
                        text_x = 10;
                        text_y += 16;
                    }
                }
            }

            // Draw cursor block
            compositor.draw_rect(text_x, text_y, 8, 8, 255, 255, 255);

            // 3. Draw mouse pointer (simple red square for now)
            compositor.draw_rect(mx, my, 4, 4, 255, 0, 0);

            // 4. Flip buffers to screen
            compositor.swap_buffers();

            // Halt until next interrupt to save CPU cycles
            x86_64::instructions::hlt();
        }
    }

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::int3();
    loop {
        x86_64::instructions::hlt();
    }
}
