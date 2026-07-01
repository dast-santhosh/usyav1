use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use crate::font::{BASIC_FONT, FONT_HEIGHT, FONT_WIDTH};

pub struct Compositor<'a> {
    front_buffer: &'a mut [u8],
    back_buffer: &'a mut [u8],
    info: FrameBufferInfo,
}

impl<'a> Compositor<'a> {
    pub fn new(framebuffer: &'a mut FrameBuffer, back_buffer: &'a mut [u8]) -> Self {
        let info = framebuffer.info();
        Compositor {
            front_buffer: framebuffer.buffer_mut(),
            back_buffer,
            info,
        }
    }

    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        let bytes_per_pixel = self.info.bytes_per_pixel;
        for i in (0..self.back_buffer.len()).step_by(bytes_per_pixel) {
            if i + 2 < self.back_buffer.len() {
                match self.info.pixel_format {
                    PixelFormat::Rgb => {
                        self.back_buffer[i] = r;
                        self.back_buffer[i + 1] = g;
                        self.back_buffer[i + 2] = b;
                    }
                    PixelFormat::Bgr => {
                        self.back_buffer[i] = b;
                        self.back_buffer[i + 1] = g;
                        self.back_buffer[i + 2] = r;
                    }
                    PixelFormat::U8 => {
                        // Grayscale approximation
                        self.back_buffer[i] = r / 3 + g / 3 + b / 3;
                    }
                    _ => {}
                }
                if bytes_per_pixel == 4 {
                    self.back_buffer[i + 3] = 255;
                }
            }
        }
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let byte_offset = y * self.info.stride * self.info.bytes_per_pixel + x * self.info.bytes_per_pixel;
        
        if byte_offset + 2 < self.back_buffer.len() {
            match self.info.pixel_format {
                PixelFormat::Rgb => {
                    self.back_buffer[byte_offset] = r;
                    self.back_buffer[byte_offset + 1] = g;
                    self.back_buffer[byte_offset + 2] = b;
                }
                PixelFormat::Bgr => {
                    self.back_buffer[byte_offset] = b;
                    self.back_buffer[byte_offset + 1] = g;
                    self.back_buffer[byte_offset + 2] = r;
                }
                PixelFormat::U8 => {
                    self.back_buffer[byte_offset] = r / 3 + g / 3 + b / 3;
                }
                _ => {}
            }
            if self.info.bytes_per_pixel == 4 {
                self.back_buffer[byte_offset + 3] = 255;
            }
        }
    }

    pub fn draw_char(&mut self, x: usize, y: usize, c: char, r: u8, g: u8, b: u8) {
        let codepoint = c as usize;
        let glyph = if codepoint < 128 {
            BASIC_FONT[codepoint]
        } else {
            BASIC_FONT[0]
        };

        for (row_idx, row) in glyph.iter().enumerate() {
            for col_idx in 0..FONT_WIDTH {
                if (*row & (1 << (FONT_WIDTH - 1 - col_idx))) != 0 {
                    self.draw_pixel(x + col_idx, y + row_idx, r, g, b);
                }
            }
        }
    }

    pub fn draw_string(&mut self, x: usize, y: usize, s: &str, r: u8, g: u8, b: u8) {
        let mut cur_x = x;
        let mut cur_y = y;

        for c in s.chars() {
            if c == '\n' {
                cur_y += FONT_HEIGHT;
                cur_x = x;
                continue;
            }

            self.draw_char(cur_x, cur_y, c, r, g, b);
            cur_x += FONT_WIDTH;

            // Wrap around
            if cur_x + FONT_WIDTH >= self.info.width {
                cur_x = x;
                cur_y += FONT_HEIGHT;
            }
        }
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, r: u8, g: u8, b: u8) {
        for dy in 0..height {
            for dx in 0..width {
                self.draw_pixel(x + dx, y + dy, r, g, b);
            }
        }
    }

    pub fn swap_buffers(&mut self) {
        let copy_len = self.front_buffer.len().min(self.back_buffer.len());
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.back_buffer.as_ptr(),
                self.front_buffer.as_mut_ptr(),
                copy_len,
            );
        }
    }

    pub fn width(&self) -> usize {
        self.info.width
    }

    pub fn height(&self) -> usize {
        self.info.height
    }
}
