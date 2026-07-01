use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
        PageTableFlags as Flags,
    },
    PhysAddr, VirtAddr,
};

/// Initialize a new OffsetPageTable.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// Identity maps the first 2GB of physical memory using 2MiB Huge Pages.
pub fn identity_map_2gb(
    mapper: &mut impl Mapper<x86_64::structures::paging::Size2MiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let flags = Flags::PRESENT | Flags::WRITABLE | Flags::HUGE_PAGE;
    
    // Map 2GB (1024 frames of 2MiB)
    for frame_addr in (0..0x8000_0000).step_by(2 * 1024 * 1024) {
        let phys_frame = PhysFrame::containing_address(PhysAddr::new(frame_addr));
        let page = Page::containing_address(VirtAddr::new(frame_addr));

        unsafe {
            if let Ok(mapping) = mapper.map_to(page, phys_frame, flags, frame_allocator) {
                mapping.flush();
            }
        }
    }
}
