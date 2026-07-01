#![no_std]

/// The Universal Driver Framework
/// Every driver must implement this trait to be managed by the kernel.
pub trait Driver {
    /// Initialize the driver. Returns Ok(()) on success, or an error string.
    fn init(&mut self) -> Result<(), &'static str>;

    /// Return the name of the driver for logging purposes.
    fn name(&self) -> &'static str;

    /// Handle a hardware interrupt intended for this driver.
    fn handle_interrupt(&mut self);
}
