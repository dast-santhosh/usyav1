use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star, SFMask, KernelGsBase};
use x86_64::VirtAddr;
use hal::gdt::GDT;
use core::arch::global_asm;
use core::sync::atomic::{AtomicU64, Ordering};

static USER_MMAP_START: AtomicU64 = AtomicU64::new(0x0000_1000_0000_0000);

#[repr(C)]
pub struct PerCpu {
    kernel_stack: u64,
    user_stack: u64,
}

pub static mut PER_CPU: PerCpu = PerCpu {
    kernel_stack: 0,
    user_stack: 0,
};

static mut SYSCALL_STACK: [u8; 4096 * 4] = [0; 4096 * 4];

pub fn init() {
    unsafe {
        PER_CPU.kernel_stack = (&raw mut SYSCALL_STACK as *mut u8 as u64) + (4096 * 4);
        KernelGsBase::write(VirtAddr::new(&raw const PER_CPU as *const _ as u64));
    }

    // Enable System Call Extensions
    unsafe {
        Efer::update(|flags| flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
    }

    // Set STAR MSR
    // STAR[47:32] = Kernel CS/SS base
    // STAR[63:48] = User CS/SS base
    let kernel_cs = GDT.1.kernel_code_selector.0;
    let user_cs_32 = GDT.1.user_data_selector.0; // dummy
    
    // The x86_64 crate handles the bit shifting for us
    Star::write(
        GDT.1.user_code_selector, // User CS 32? Wait, x86_64 crate expects User CS 32-bit.
        GDT.1.user_data_selector,
        GDT.1.kernel_code_selector,
        GDT.1.kernel_data_selector,
    ).unwrap();

    // Set LSTAR to our assembly entry point
    LStar::write(VirtAddr::new(syscall_entry as *const () as usize as u64));

    // Clear Interrupt Flag on syscall
    use x86_64::registers::rflags::RFlags;
    SFMask::write(RFlags::INTERRUPT_FLAG);
}

#[repr(C)]
#[derive(Debug)]
pub struct SyscallRegs {
    pub rax: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub r10: u64,
    pub r8: u64,
    pub r9: u64,
    pub rsp: u64,
    pub rflags: u64,
    pub rip: u64,
}

global_asm!(
    r#"
    .global syscall_entry
    syscall_entry:
        swapgs
        
        // Save user stack and load kernel stack
        mov gs:[8], rsp
        mov rsp, gs:[0]
        
        // Build SyscallRegs on the stack
        push rcx // rip (rcx contains rip from syscall instr)
        push r11 // rflags (r11 contains rflags from syscall instr)
        push gs:[8] // rsp
        push r9
        push r8
        push r10
        push rdx
        push rsi
        push rdi
        push rax
        
        // Pass pointer to SyscallRegs as the first argument
        mov rdi, rsp
        
        // Stack must be 16-byte aligned before call
        // Currently we pushed 10 registers (80 bytes), plus the hardware didn't push anything.
        // Wait, 80 bytes is a multiple of 16! So the stack IS 16-byte aligned! (Assuming kernel stack was 16-byte aligned to begin with).
        
        call syscall_handler_rust
        
        // The return value is in rax, which we'll write into the SyscallRegs so it gets popped correctly?
        // Actually, pop rax will overwrite whatever we do.
        // We can just mov [rsp], rax so that `pop rax` loads the return value!
        mov [rsp], rax
        
        // Restore registers
        pop rax
        pop rdi
        pop rsi
        pop rdx
        pop r10
        pop r8
        pop r9
        
        // Restore user stack and sysret
        pop rsp // this was gs:[8], but we can just pop it to some scratch? Wait! We can't pop to rsp directly here because we still need to pop rflags and rip!
        // Instead:
        add rsp, 8 // skip rsp
        pop r11
        pop rcx
        
        mov rsp, gs:[8]
        swapgs
        sysretq
    "#
);

extern "C" {
    fn syscall_entry();
}

#[no_mangle]
pub extern "C" fn syscall_handler_rust(regs: &mut SyscallRegs) -> u64 {
    match regs.rax {
        0 => { // read
            log::info!("Syscall: read({}, {}, {})", regs.rdi, regs.rsi, regs.rdx);
            0
        }
        1 => { // write
            let fd = regs.rdi;
            let ptr = regs.rsi as *const u8;
            let len = regs.rdx as usize;
            
            if fd == 1 || fd == 2 {
                // stdout / stderr
                unsafe {
                    let slice = core::slice::from_raw_parts(ptr, len);
                    for &b in slice {
                        let mut lsr: u8;
                        loop {
                            core::arch::asm!("in al, dx", out("al") lsr, in("dx") 0x3FD_u16, options(nomem, nostack, preserves_flags));
                            if (lsr & 0x20) != 0 {
                                break;
                            }
                        }
                        core::arch::asm!("out dx, al", in("dx") 0x3F8_u16, in("al") b, options(nomem, nostack, preserves_flags));
                    }
                }
                len as u64
            } else {
                !0 // -1
            }
        }
        2 => { // open
            log::info!("Syscall: open");
            !0 // -1
        }
        9 => { // mmap
            let size = regs.rdi;
            log::info!("Syscall: mmap(size={})", size);
            if size == 0 {
                return !0; // -1
            }
            
            let num_pages = (size + 4095) / 4096;
            let mut allocated_addr = !0; // Default to -1 (fail)

            // Disable interrupts to prevent spinlock deadlock during page mapping
            x86_64::instructions::interrupts::without_interrupts(|| {
                let mut mem_guard = kernel::memory::MEMORY_MANAGER.lock();
                if let Some(mem) = mem_guard.as_mut() {
                    use x86_64::structures::paging::{Page, PageTableFlags as Flags, Mapper, FrameAllocator};
                    use x86_64::VirtAddr;
                    
                    let start_addr = USER_MMAP_START.fetch_add(num_pages * 4096, Ordering::SeqCst);
                    let mut success = true;

                    for i in 0..num_pages {
                        let page = Page::containing_address(VirtAddr::new(start_addr + i * 4096));
                        
                        // Allocate a physical frame
                        if let Some(frame) = mem.frame_allocator.allocate_frame() {
                            let flags = Flags::PRESENT | Flags::WRITABLE | Flags::USER_ACCESSIBLE;
                            
                            // Map the virtual page to the physical frame
                            unsafe {
                                if let Ok(mapping) = mem.mapper.map_to(page, frame, flags, &mut mem.frame_allocator) {
                                    mapping.flush(); // Invalidate TLB for the new mapping
                                } else {
                                    success = false;
                                    break;
                                }
                            }
                        } else {
                            success = false;
                            break;
                        }
                    }

                    if success {
                        allocated_addr = start_addr;
                    }
                }
            });

            allocated_addr
        }
        11 => { // munmap
            log::info!("Syscall: munmap(addr={:#x})", regs.rdi);
            // TODO: Implement page unmapping
            0
        }
        56 => { // clone
            log::info!("Syscall: clone(fn={:#x}, arg={:#x})", regs.rdi, regs.rsi);
            // TODO: Implement thread spawning and TSS setup
            !0 // -1 (fail)
        }
        60 => { // exit
            log::info!("Syscall: exit({})", regs.rdi);
            loop {
                x86_64::instructions::hlt();
            }
        }
        _ => {
            log::warn!("Unknown syscall: {}", regs.rax);
            !0
        }
    }
}
