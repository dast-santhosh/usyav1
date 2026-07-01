use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, OffsetPageTable
};
use x86_64::{PhysAddr, VirtAddr};
use xmas_elf::{ElfFile, program::{ProgramHeader, Type}};

pub fn start_process(
    elf_data: &[u8],
    physical_memory_offset: u64,
) {
    let elf = ElfFile::new(elf_data).expect("Failed to parse ELF");

    let (new_p4_frame, user_rsp, user_rip) = {
        let mut mem_guard = kernel::memory::MEMORY_MANAGER.lock();
        let mem = mem_guard.as_mut().expect("Global memory manager not initialized");
        
        // 1. Create a new Level 4 Page Table for the process
        let new_p4_frame = mem.frame_allocator.allocate_frame().expect("No frames available for P4");
        let new_p4_addr = new_p4_frame.start_address().as_u64() + physical_memory_offset;
        let new_p4 = unsafe { &mut *(new_p4_addr as *mut PageTable) };
        
        // Clear the new table
        new_p4.zero();

        // Copy kernel mappings from current CR3 to the new P4 table
        use x86_64::registers::control::Cr3;
        let (current_p4_frame, _) = Cr3::read();
        let current_p4_addr = current_p4_frame.start_address().as_u64() + physical_memory_offset;
        let current_p4 = unsafe { &*(current_p4_addr as *const PageTable) };
        
        for i in 0..512 {
            new_p4[i] = current_p4[i].clone();
        }

        let mut process_mapper = unsafe { OffsetPageTable::new(new_p4, VirtAddr::new(physical_memory_offset)) };

        // 2. Map ELF segments
        for ph in elf.program_iter() {
            if let Type::Load = ph.get_type().unwrap() {
                let start_addr = VirtAddr::new(ph.virtual_addr());
                let start_page: Page = Page::containing_address(start_addr);
                let end_page: Page = Page::containing_address(start_addr + ph.mem_size() - 1u64);

                let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;

                for page in Page::range_inclusive(start_page, end_page) {
                    let frame = mem.frame_allocator.allocate_frame().unwrap();
                    
                    // Map it
                    unsafe {
                        process_mapper.map_to(page, frame, flags, &mut mem.frame_allocator)
                            .expect("Failed to map ELF page")
                            .flush();
                    }

                    // Copy data if within file_size
                    let page_offset = page.start_address().as_u64() - start_addr.as_u64();
                    if page_offset < ph.file_size() {
                        let copy_size = core::cmp::min(4096, ph.file_size() - page_offset) as usize;
                        let src = &elf_data[(ph.offset() + page_offset) as usize .. (ph.offset() + page_offset) as usize + copy_size];
                        
                        // We must write to the physical frame via the identity offset
                        let dst_addr = frame.start_address().as_u64() + physical_memory_offset;
                        let dst = unsafe { core::slice::from_raw_parts_mut(dst_addr as *mut u8, copy_size) };
                        dst.copy_from_slice(src);
                        
                        // Zero the rest of the page (BSS)
                        if copy_size < 4096 {
                            let bss = unsafe { core::slice::from_raw_parts_mut((dst_addr + copy_size as u64) as *mut u8, 4096 - copy_size) };
                            bss.fill(0);
                        }
                    } else {
                        // Entirely BSS
                        let dst_addr = frame.start_address().as_u64() + physical_memory_offset;
                        let dst = unsafe { core::slice::from_raw_parts_mut(dst_addr as *mut u8, 4096) };
                        dst.fill(0);
                    }
                }
            }
        }

        // 3. Map User Stack
        let stack_start = Page::containing_address(VirtAddr::new(0x0000_7FFF_FFFF_0000));
        let stack_end = Page::containing_address(VirtAddr::new(0x0000_7FFF_FFFF_F000)); // 16 pages
        for page in Page::range_inclusive(stack_start, stack_end) {
            let frame = mem.frame_allocator.allocate_frame().unwrap();
            let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;
            unsafe {
                process_mapper.map_to(page, frame, flags, &mut mem.frame_allocator)
                    .expect("Failed to map user stack")
                    .flush();
            }
        }

        let user_rsp = 0x0000_7FFF_FFFF_F000u64 + 4096;
        let user_rip = elf.header.pt2.entry_point();
        
        (new_p4_frame, user_rsp, user_rip)
    }; // Lock is dropped here!

    log::info!("Jumping to Ring 3! RIP: {:#x}, RSP: {:#x}", user_rip, user_rsp);

    // 4. Switch CR3 and Jump to Ring 3 via iretq
    use x86_64::registers::control::Cr3;
    unsafe {
        Cr3::write(new_p4_frame, Cr3::read().1);

        // We use iretq to switch to Ring 3.
        // We need:
        // SS, RSP, RFLAGS, CS, RIP
        
        let user_data_selector = hal::gdt::GDT.1.user_data_selector.0;
        let user_code_selector = hal::gdt::GDT.1.user_code_selector.0;

        core::arch::asm!(
            "push {ss}",
            "push {rsp}",
            "push 0x202", // RFLAGS with interrupts enabled (IF=1)
            "push {cs}",
            "push {rip}",
            "iretq",
            ss = in(reg) user_data_selector as u64,
            rsp = in(reg) user_rsp,
            cs = in(reg) user_code_selector as u64,
            rip = in(reg) user_rip,
            options(noreturn)
        );
    }
}
