#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb888,
    prelude::*,
};
use tinybmp::Bmp;

entry_point!(kernel_main);

struct Display<'a> {
    framebuffer: &'a mut bootloader_api::info::FrameBuffer,
}

impl<'a> DrawTarget for Display<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let info = self.framebuffer.info();
        let stride = info.stride;
        let bytes_per_pixel = info.bytes_per_pixel;
        let width = info.width;
        let height = info.height;
        let buffer = self.framebuffer.buffer_mut();

        for Pixel(Point { x, y }, color) in pixels.into_iter() {
            if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
                let pixel_offset = ((y as usize) * stride + (x as usize)) * bytes_per_pixel;
                if pixel_offset + bytes_per_pixel <= buffer.len() {
                    match info.pixel_format {
                        bootloader_api::info::PixelFormat::Rgb => {
                            buffer[pixel_offset] = color.r();
                            buffer[pixel_offset + 1] = color.g();
                            buffer[pixel_offset + 2] = color.b();
                        },
                        bootloader_api::info::PixelFormat::Bgr => {
                            buffer[pixel_offset] = color.b();
                            buffer[pixel_offset + 1] = color.g();
                            buffer[pixel_offset + 2] = color.r();
                        },
                        bootloader_api::info::PixelFormat::U8 => {
                            let gray = ((color.r() as u16 + color.g() as u16 + color.b() as u16) / 3) as u8;
                            buffer[pixel_offset] = gray;
                        },
                        _ => {}
                    }
                    // If it's a 32-bit format (4 bytes per pixel), set the 4th byte to 255 (opaque alpha)
                    if bytes_per_pixel == 4 {
                        buffer[pixel_offset + 3] = 255;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        let info = self.framebuffer.info();
        Size::new(info.width as u32, info.height as u32)
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let mut display = Display { framebuffer };

        // Clear the screen to pure white
        let _ = display.clear(Rgb888::WHITE);

        // Initialize HAL (GDT, IDT, PIC)
        hal::init();

        // Initialize Memory Subsystem (Paging, PMM)
        let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
        let mut mapper = unsafe { kernel::memory::init(phys_mem_offset) };
        let mut frame_allocator = unsafe { kernel::memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

        // Clear the screen to pure white
        let _ = display.clear(Rgb888::WHITE);

        // Parse and embed the BMP logo
        let bmp_data = include_bytes!("assets/logo.bmp");
        match Bmp::<Rgb888>::from_slice(bmp_data) {
            Ok(bmp) => {
                let display_size = display.size();
                let bmp_size = bmp.bounding_box().size;
                
                // Calculate mathematical center
                let center_x = (display_size.width.saturating_sub(bmp_size.width)) / 2;
                let center_y = (display_size.height.saturating_sub(bmp_size.height)) / 2;
                
                // Draw the image
                let _ = Image::new(&bmp, Point::new(center_x as i32, center_y as i32))
                    .draw(&mut display);
            }
            Err(_) => {
                // VISUAL DEBUGGING: If the BMP fails to parse, paint the screen RED
                let _ = display.clear(Rgb888::RED);
            }
        }
    }

    loop {
        // Halt to save CPU cycles on x86_64
        #[cfg(target_arch = "x86_64")]
        unsafe { core::arch::asm!("hlt") };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Ideally we would draw the panic string to the framebuffer here.
    // For now, we will just issue a software breakpoint.
    x86_64::instructions::interrupts::int3();
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe { core::arch::asm!("hlt") };
    }
}
