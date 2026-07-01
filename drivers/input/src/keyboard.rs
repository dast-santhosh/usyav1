use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

const PS2_DATA_PORT: u16 = 0x60;

lazy_static! {
    pub static ref KEYBOARD_QUEUE: Mutex<RingBuffer<256>> = Mutex::new(RingBuffer::new());
}

pub struct RingBuffer<const N: usize> {
    buffer: [char; N],
    head: usize,
    tail: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        RingBuffer {
            buffer: ['\0'; N],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, item: char) {
        let next = (self.head + 1) % N;
        if next != self.tail {
            self.buffer[self.head] = item;
            self.head = next;
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.head == self.tail {
            None
        } else {
            let item = self.buffer[self.tail];
            self.tail = (self.tail + 1) % N;
            Some(item)
        }
    }
}

pub fn handle_interrupt() {
    let mut data_port = Port::<u8>::new(PS2_DATA_PORT);
    let scancode = unsafe { data_port.read() };
    
    // Very basic Set 1 QWERTY mapping (lowercase only for now, plus backspace and enter)
    let c = match scancode {
        0x02 => Some('1'), 0x03 => Some('2'), 0x04 => Some('3'), 0x05 => Some('4'),
        0x06 => Some('5'), 0x07 => Some('6'), 0x08 => Some('7'), 0x09 => Some('8'),
        0x0A => Some('9'), 0x0B => Some('0'),
        
        0x10 => Some('q'), 0x11 => Some('w'), 0x12 => Some('e'), 0x13 => Some('r'),
        0x14 => Some('t'), 0x15 => Some('y'), 0x16 => Some('u'), 0x17 => Some('i'),
        0x18 => Some('o'), 0x19 => Some('p'),
        
        0x1E => Some('a'), 0x1F => Some('s'), 0x20 => Some('d'), 0x21 => Some('f'),
        0x22 => Some('g'), 0x23 => Some('h'), 0x24 => Some('j'), 0x25 => Some('k'),
        0x26 => Some('l'),
        
        0x2C => Some('z'), 0x2D => Some('x'), 0x2E => Some('c'), 0x2F => Some('v'),
        0x30 => Some('b'), 0x31 => Some('n'), 0x32 => Some('m'),
        
        0x39 => Some(' '), // Space
        0x1C => Some('\n'), // Enter
        0x0E => Some('\x08'), // Backspace
        _ => None,
    };

    if let Some(ch) = c {
        KEYBOARD_QUEUE.lock().push(ch);
    }
}
