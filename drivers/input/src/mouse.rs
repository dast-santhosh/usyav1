use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;

lazy_static! {
    pub static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
}

pub struct Mouse {
    data_port: Port<u8>,
    command_port: Port<u8>,
    byte_count: u8,
    packet: [u8; 3],
    pub x: i32,
    pub y: i32,
}

impl Mouse {
    pub const fn new() -> Self {
        Mouse {
            data_port: Port::new(0x60),
            command_port: Port::new(0x64),
            byte_count: 0,
            packet: [0; 3],
            x: 0,
            y: 0,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            // Enable aux device
            self.wait_write();
            self.command_port.write(0xA8);

            // Read compaq status byte
            self.wait_write();
            self.command_port.write(0x20);
            self.wait_read();
            let mut status = self.data_port.read();
            
            // Enable IRQ12
            status |= 1 << 1;
            status &= !(1 << 5);
            
            self.wait_write();
            self.command_port.write(0x60);
            self.wait_write();
            self.data_port.write(status);

            // Enable mouse packets
            self.write_mouse(0xF4);
        }
    }

    unsafe fn write_mouse(&mut self, data: u8) {
        self.wait_write();
        self.command_port.write(0xD4);
        self.wait_write();
        self.data_port.write(data);
        self.wait_read();
        let _ack = self.data_port.read(); // Read ACK
    }

    unsafe fn wait_write(&mut self) {
        for _ in 0..10000 {
            if (self.command_port.read() & 2) == 0 {
                return;
            }
        }
    }

    unsafe fn wait_read(&mut self) {
        for _ in 0..10000 {
            if (self.command_port.read() & 1) == 1 {
                return;
            }
        }
    }

    pub fn handle_interrupt(&mut self) {
        unsafe {
            let data = self.data_port.read();
            
            if self.byte_count == 0 && (data & 0x08) == 0 {
                return; // Sync error
            }

            self.packet[self.byte_count as usize] = data;
            self.byte_count += 1;

            if self.byte_count == 3 {
                self.byte_count = 0;

                let dx = self.packet[1] as i8 as i32;
                let dy = self.packet[2] as i8 as i32;

                // Update coordinates
                self.x = self.x.saturating_add(dx).clamp(0, 1920);
                self.y = self.y.saturating_sub(dy).clamp(0, 1080); // Y is usually inverted
            }
        }
    }
}
