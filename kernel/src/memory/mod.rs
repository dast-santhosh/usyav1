pub mod pmm;
pub mod paging;
pub mod heap;

use spin::Mutex;
use x86_64::structures::paging::OffsetPageTable;
use crate::memory::pmm::BitmapFrameAllocator;

pub struct MemoryManager {
    pub mapper: OffsetPageTable<'static>,
    pub frame_allocator: BitmapFrameAllocator,
}

pub static MEMORY_MANAGER: Mutex<Option<MemoryManager>> = Mutex::new(None);

pub fn init_global_memory(mapper: OffsetPageTable<'static>, frame_allocator: BitmapFrameAllocator) {
    *MEMORY_MANAGER.lock() = Some(MemoryManager {
        mapper,
        frame_allocator,
    });
}
