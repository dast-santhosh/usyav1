use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

/// 2 GB max physical memory tracking
const MAX_FRAMES: usize = 524_288;
const BITMAP_SIZE: usize = MAX_FRAMES / 64;

pub struct BitmapFrameAllocator {
    bitmap: [u64; BITMAP_SIZE],
    next_free: usize,
}

impl BitmapFrameAllocator {
    /// Creates an empty frame allocator.
    pub const fn new() -> Self {
        Self {
            bitmap: [0; BITMAP_SIZE],
            next_free: 0,
        }
    }

    /// Initializes the frame allocator with the usable regions from the bootloader memory map.
    pub fn init(&mut self, memory_regions: &'static MemoryRegions) {
        // Mark everything as used initially (1 = used, 0 = free)
        for i in 0..BITMAP_SIZE {
            self.bitmap[i] = !0;
        }

        // Iterate over usable regions and mark them as free
        for region in memory_regions.iter() {
            if region.kind == MemoryRegionKind::Usable {
                let start_frame = (region.start / 4096) as usize;
                let end_frame = (region.end / 4096) as usize;
                
                for frame in start_frame..end_frame {
                    if frame < MAX_FRAMES {
                        self.mark_free(frame);
                    }
                }
            }
        }
    }

    fn mark_free(&mut self, frame: usize) {
        let index = frame / 64;
        let bit = frame % 64;
        self.bitmap[index] &= !(1 << bit);
    }

    fn mark_used(&mut self, frame: usize) {
        let index = frame / 64;
        let bit = frame % 64;
        self.bitmap[index] |= 1 << bit;
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Simple linear search starting from next_free
        for i in self.next_free..BITMAP_SIZE {
            let chunk = self.bitmap[i];
            if chunk != !0 { // There is at least one free frame (a 0 bit)
                let bit = chunk.trailing_ones() as usize;
                let frame = i * 64 + bit;
                self.mark_used(frame);
                self.next_free = i; // Save progress
                return Some(PhysFrame::containing_address(PhysAddr::new((frame * 4096) as u64)));
            }
        }
        
        // If we reach here, check from the beginning up to next_free
        for i in 0..self.next_free {
            let chunk = self.bitmap[i];
            if chunk != !0 {
                let bit = chunk.trailing_ones() as usize;
                let frame = i * 64 + bit;
                self.mark_used(frame);
                self.next_free = i;
                return Some(PhysFrame::containing_address(PhysAddr::new((frame * 4096) as u64)));
            }
        }

        None
    }
}
