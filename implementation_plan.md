# Phase 3: Universal Driver Framework & NVMe Driver

This plan outlines the architecture for introducing the Universal Driver Framework and implementing the NVMe driver by reading the PCI configuration space.

## User Review Required

> [!WARNING]  
> **ISO Generation Constraint:** The modern `bootloader` v0.11 API removed native support for generating `.iso` files (which was present in v0.9 via the `bootimage` crate). It only generates raw disk images (`.bin`) and UEFI FAT images (`.efi`). To generate a true `.iso` on Windows, we would need to invoke external tools like `xorriso` or `oscdimg` from our `xtask` builder. 
> 
> **Recommendation:** We can either leave it as `.bin` since `qemu -drive format=raw,file=bootimage-bios.bin` boots it perfectly, or we can add a PowerShell script to bundle it into an ISO. Please let me know how you'd like to proceed with the ISO requirement!

## Proposed Changes

### 1. Universal Driver Framework
#### [MODIFY] [drivers/src/lib.rs](file:///e:/GRAY/usyav1/drivers/src/lib.rs)
- Define the `Driver` trait:
  ```rust
  pub trait Driver {
      fn init(&mut self) -> Result<(), &'static str>;
      fn name(&self) -> &'static str;
      fn handle_interrupt(&mut self);
  }
  ```

### 2. PCI Bus Enumerator
#### [NEW] [hal/src/pci.rs](file:///e:/GRAY/usyav1/hal/src/pci.rs)
- Create a PCI configuration space reader using `x86_64::instructions::port::Port`.
- Implement `read_config_dword(bus, slot, func, offset)`.
- Implement a `scan_bus()` function that loops through Bus 0, Slot 0-31, Func 0-7, and reads the Vendor ID and Device ID, logging them to the serial port.

#### [MODIFY] [hal/src/lib.rs](file:///e:/GRAY/usyav1/hal/src/lib.rs)
- Expose the new `pci` module.

### 3. NVMe Storage Driver
#### [MODIFY] [drivers/storage/src/lib.rs](file:///e:/GRAY/usyav1/drivers/storage/src/lib.rs)
- Define `NvmeController` implementing the `Driver` trait.
- Implement the `init()` method that:
  1. Searches the PCI bus for the NVMe Class Code (`0x01` mass storage, `0x08` non-volatile).
  2. Reads the PCI BAR0 to locate the NVMe memory-mapped registers.
  3. Maps the BAR0 physical address to a virtual address.
  4. Includes a Watchdog Timer: a loop up to 5,000,000 cycles checking for device readiness (e.g., `CSTS.RDY`), returning `NvmeError::Timeout` if it fails instead of hanging.

## Verification Plan

### Automated Tests
- Run `cargo xtask` to ensure the new PCI and NVMe logic compiles correctly.

### Manual Verification
- We will boot the kernel in QEMU with an attached NVMe drive (e.g., `-drive file=test.img,if=none,id=drv0 -device nvme,drive=drv0,serial=foo`).
- We will verify the serial output to confirm the PCI bus scanner detects the NVMe controller and the driver successfully initializes (or times out gracefully).
