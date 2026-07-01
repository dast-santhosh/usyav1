use x86_64::instructions::port::Port;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

pub fn read_config_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = 0x80000000 
                | ((bus as u32) << 16)
                | ((slot as u32) << 11)
                | ((func as u32) << 8)
                | ((offset as u32) & 0xFC);
                
    let mut config_addr_port: Port<u32> = Port::new(CONFIG_ADDRESS);
    let mut config_data_port: Port<u32> = Port::new(CONFIG_DATA);

    unsafe {
        config_addr_port.write(address);
        config_data_port.read()
    }
}

pub fn scan_bus() {
    log::info!("Scanning PCI Bus...");
    // Just scan bus 0 for now
    let bus = 0;
    for slot in 0..32 {
        for func in 0..8 {
            let vendor_device = read_config_dword(bus, slot, func, 0);
            let vendor_id = (vendor_device & 0xFFFF) as u16;
            let device_id = (vendor_device >> 16) as u16;

            if vendor_id != 0xFFFF {
                log::info!("PCI Device found at {}:{}:{} - Vendor: {:#06X}, Device: {:#06X}", bus, slot, func, vendor_id, device_id);
            }
        }
    }
}
