use core::fmt;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub struct SerialPort {
    data: Port<u8>,
    int_en: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_sts: Port<u8>,
}

impl SerialPort {
    pub const fn new(port: u16) -> SerialPort {
        SerialPort {
            data: Port::new(port),
            int_en: Port::new(port + 1),
            fifo_ctrl: Port::new(port + 2),
            line_ctrl: Port::new(port + 3),
            modem_ctrl: Port::new(port + 4),
            line_sts: Port::new(port + 5),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            // Disable all interrupts
            self.int_en.write(0x00);
            // Enable DLAB (set baud rate divisor)
            self.line_ctrl.write(0x80);
            // Set divisor to 3 (lo byte) 38400 baud
            self.data.write(0x03);
            self.int_en.write(0x00);
            // 8 bits, no parity, one stop bit
            self.line_ctrl.write(0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            self.fifo_ctrl.write(0xC7);
            // IRQs enabled, RTS/DSR set
            self.modem_ctrl.write(0x0B);
            // Enable interrupts
            self.int_en.write(0x01);
        }
    }

    fn wait_for_tx_empty(&mut self) {
        unsafe {
            while (self.line_sts.read() & 0x20) == 0 {
                core::hint::spin_loop();
            }
        }
    }

    pub fn send(&mut self, data: u8) {
        match data {
            8 | 0x7F => unsafe {
                self.wait_for_tx_empty();
                self.data.write(8);
                self.wait_for_tx_empty();
                self.data.write(b' ');
                self.wait_for_tx_empty();
                self.data.write(8);
            },
            _ => unsafe {
                self.wait_for_tx_empty();
                self.data.write(data);
            },
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

pub static SERIAL1: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x3F8));

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    });
}
