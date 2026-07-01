#![no_std]

use core::ptr::{read_volatile, write_volatile};

/// Generic Storage Driver trait implemented by all block devices (NVMe, AHCI, etc.)
pub trait StorageDriver {
    /// Read a block of data from the storage device.
    fn read_block(&mut self, lba: u64, buffer: &mut [u8]) -> Result<(), &'static str>;
    
    /// Write a block of data to the storage device.
    fn write_block(&mut self, lba: u64, buffer: &[u8]) -> Result<(), &'static str>;
    
    /// Returns the total capacity of the device in blocks.
    fn capacity_blocks(&self) -> u64;
}

pub enum NvmeError {
    NoControllerFound,
    InvalidBar0,
    UnsupportedCommandSet,
    Timeout,
}

/// A structure representing an NVMe Controller mapped in memory
pub struct NvmeController {
    _bus: u8,
    _device: u8,
    _function: u8,
    bar0: usize,
    /// Indicates if the controller is initialized and ready for commands
    ready: bool,
}

impl NvmeController {
    /// Initialize the NVMe Controller by scanning the PCI bus.
    pub fn init_controller() -> Result<Self, NvmeError> {
        // Scan PCI bus for a device with Class 0x01 (Mass Storage), 
        // Subclass 0x08 (Non-Volatile Memory), ProgIF 0x02 (NVM Express)
        // Optimize for QEMU: Only scan Bus 0 to avoid massive I/O port emulation lag
        for bus in 0..=0 {
            for device in 0..32 {
                for function in 0..8 {
                    let vendor_id = Self::pci_read_u16(bus, device, function, 0x00);
                    if vendor_id == 0xFFFF {
                        continue; // No device present at this address
                    }

                    let class_subclass = Self::pci_read_u16(bus, device, function, 0x0A);
                    let base_class = (class_subclass >> 8) as u8;
                    let sub_class = (class_subclass & 0xFF) as u8;
                    let prog_if = (Self::pci_read_u16(bus, device, function, 0x08) >> 8) as u8;

                    // Match NVMe specifications
                    if base_class == 0x01 && sub_class == 0x08 && prog_if == 0x02 {
                        // Found NVMe controller! Read BAR0 for memory-mapped I/O base address
                        let bar0_low = Self::pci_read_u32(bus, device, function, 0x10);
                        let bar0_high = Self::pci_read_u32(bus, device, function, 0x14);
                        
                        // Mask out the lower 4 bits (flags) to get the physical 64-bit address
                        let bar0_addr = ((bar0_high as usize) << 32) | (bar0_low as usize & !0xF);
                        
                        let mut controller = NvmeController {
                            _bus: bus,
                            _device: device,
                            _function: function,
                            bar0: bar0_addr,
                            ready: false,
                        };

                        controller.configure_controller()?;
                        return Ok(controller);
                    }
                }
            }
        }
        
        Err(NvmeError::NoControllerFound)
    }

    /// Internal method to configure the NVMe controller registers
    fn configure_controller(&mut self) -> Result<(), NvmeError> {
        if self.bar0 == 0 {
            return Err(NvmeError::InvalidBar0);
        }

        unsafe {
            // Read Controller Capabilities (CAP) register at offset 0x00
            let _cap_low = read_volatile((self.bar0 + 0x00) as *const u32);
            let cap_high = read_volatile((self.bar0 + 0x04) as *const u32);
            
            // Check if the controller supports the NVM command set (CAP.CSS bit 37)
            if (cap_high & (1 << 5)) == 0 {
                return Err(NvmeError::UnsupportedCommandSet);
            }

            // Disable the controller before configuring (Clear CC.EN, bit 0)
            let mut cc = read_volatile((self.bar0 + 0x14) as *mut u32);
            cc &= !1; 
            write_volatile((self.bar0 + 0x14) as *mut u32, cc);

            // Wait for CSTS.RDY (bit 0) to become 0, indicating disabled state
            while (read_volatile((self.bar0 + 0x1C) as *const u32) & 1) != 0 {
                core::hint::spin_loop();
            }

            // Configure Admin Queue Attributes (AQA) at offset 0x24
            // Example: 64 entries for Admin Submission Queue (ASQ) and Admin Completion Queue (ACQ)
            // (Values are 0-based, so 63 = 64 entries)
            let aqa = (63 << 16) | 63;
            write_volatile((self.bar0 + 0x24) as *mut u32, aqa);

            // Note: In a fully functional environment, we would allocate physical memory 
            // for the ASQ (0x28) and ACQ (0x30) and write their addresses here.

            // Enable controller (Set CC.EN)
            cc |= 1;
            // Select NVM Command Set (CSS = 0b000)
            cc &= !(7 << 4);
            write_volatile((self.bar0 + 0x14) as *mut u32, cc);

            // Wait for CSTS.RDY to become 1, indicating readiness
            let mut timeout = 5_000_000;
            while (read_volatile((self.bar0 + 0x1C) as *const u32) & 1) == 0 {
                timeout -= 1;
                if timeout == 0 {
                    return Err(NvmeError::Timeout);
                }
                core::hint::spin_loop();
            }
        }

        self.ready = true;
        Ok(())
    }

    /// Helper to read 32-bit value from PCI Configuration Space using Legacy Port I/O
    fn pci_read_u32(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
        let address = 0x80000000 
            | ((bus as u32) << 16) 
            | ((device as u32) << 11) 
            | ((func as u32) << 8) 
            | ((offset as u32) & 0xFC);
            
        unsafe {
            Self::outl(0xCF8, address);
            Self::inl(0xCFC)
        }
    }

    /// Helper to read 16-bit value from PCI Configuration Space
    fn pci_read_u16(bus: u8, device: u8, func: u8, offset: u8) -> u16 {
        let val = Self::pci_read_u32(bus, device, func, offset);
        ((val >> ((offset & 2) * 8)) & 0xFFFF) as u16
    }

    /// Low-level Port I/O: Output 32-bit double word
    #[inline]
    unsafe fn outl(port: u16, value: u32) {
        #[cfg(target_arch = "x86_64")]
        unsafe { core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags)); }
        #[cfg(not(target_arch = "x86_64"))]
        { let _ = (port, value); }
    }

    /// Low-level Port I/O: Input 32-bit double word
    #[inline]
    unsafe fn inl(port: u16) -> u32 {
        #[cfg(target_arch = "x86_64")]
        {
            let value: u32;
            unsafe { core::arch::asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags)); }
            value
        }
        #[cfg(not(target_arch = "x86_64"))]
        { let _ = port; 0 }
    }
}

impl StorageDriver for NvmeController {
    fn read_block(&mut self, _lba: u64, _buffer: &mut [u8]) -> Result<(), &'static str> {
        if !self.ready {
            return Err("Controller not ready");
        }
        // Next step: Allocate physical memory for PRPs, build a Read Command (Opcode 0x02),
        // submit it to the I/O Submission Queue, ring the doorbell (offset 0x1000 + SQ * stride),
        // and poll the I/O Completion Queue.
        Ok(())
    }

    fn write_block(&mut self, _lba: u64, _buffer: &[u8]) -> Result<(), &'static str> {
        if !self.ready {
            return Err("Controller not ready");
        }
        // Next step: Similar to read_block but with Write Command (Opcode 0x01).
        Ok(())
    }

    fn capacity_blocks(&self) -> u64 {
        // Normally retrieved dynamically via Identify Namespace command.
        // Assuming a generic 1GB drive for now (512 byte blocks).
        2 * 1024 * 1024
    }
}

impl drivers::Driver for NvmeController {
    fn init(&mut self) -> Result<(), &'static str> {
        self.configure_controller().map_err(|_| "Failed to configure NVMe Controller")
    }

    fn name(&self) -> &'static str {
        "NVMe Storage Driver"
    }

    fn handle_interrupt(&mut self) {
        // Acknowledge NVMe interrupt here
    }
}
