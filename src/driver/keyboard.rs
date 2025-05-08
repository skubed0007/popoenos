extern crate x86_64;
extern crate lazy_static;

use x86_64::instructions::port::Port;
use lazy_static::lazy_static;
use spin::Mutex;
use core::option::Option::{self, None, Some};

// I/O Ports for PS/2 Keyboard
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

// Scancode Set 2 (partial, for demonstration purposes)
const SCANCODE_SET2: [char; 128] = [
    '\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\0', '\0', // 0x00 - 0x0F
    'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '\0', 'a', 's', // 0x10 - 0x1F
    'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`', '\0', '\\', 'z', 'x', 'c', 'v', // 0x20 - 0x2F
    'b', 'n', 'm', ',', '.', '/', '\0', '*', '\0', ' ', '\0', '\0', '\0', '\0', '\0', '\0', // 0x30 - 0x3F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x40 - 0x4F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x50 - 0x5F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x60 - 0x6F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x70 - 0x7F
];

// Shifted Scancode Set 2 (partial, for demonstration purposes)
const SHIFT_SCANCODE_SET2: [char; 128] = [
    '\0', '\0', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\0', '\0', // 0x00 - 0x0F
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n', '\0', 'A', 'S', // 0x10 - 0x1F
    'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~', '\0', '|', 'Z', 'X', 'C', 'V', // 0x20 - 0x2F
    'B', 'N', 'M', '<', '>', '?', '\0', '*', '\0', ' ', '\0', '\0', '\0', '\0', '\0', '\0', // 0x30 - 0x3F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x40 - 0x4F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x50 - 0x5F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x60 - 0x6F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x70 - 0x7F
];

lazy_static! {
    static ref KEYBOARD: Mutex<Option<Keyboard>> = Mutex::new(None);
}

pub struct Keyboard {
    data_port: Port<u8>,
    status_port: Port<u8>,
    shift_pressed: bool,
    caps_lock: bool,
    extended: bool, // For handling extended scancodes
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            data_port: Port::new(PS2_DATA_PORT),
            status_port: Port::new(PS2_STATUS_PORT),
            shift_pressed: false,
            caps_lock: false,
            extended: false,
        }
    }

    fn read_status(&mut self) -> u8 {
        unsafe { self.status_port.read() }
    }

    fn read_data(&mut self) -> u8 {
        unsafe { self.data_port.read() }
    }

    pub fn init(&mut self) {
        // Initialize the keyboard
    }

    pub fn get_key(&mut self) -> Option<char> {
        while self.read_status() & 0x01 == 0 {}

        let scancode = self.read_data();

        // Handle extended scancodes
        if scancode == 0xE0 {
            self.extended = true;
            return None;
        }

        // Handle key release
        if scancode & 0x80 != 0 {
            let key = scancode & 0x7F;
            if key == 0x2A || key == 0x36 {
                self.shift_pressed = false;
            }
            return None;
        }

        // Handle key press
        match scancode {
            0x2A | 0x36 => {
                self.shift_pressed = true;
                None
            }
            0x3A => {
                self.caps_lock = !self.caps_lock;
                None
            }
            0x0E => {
                // Backspace key
                Some('\x08') // Return the backspace character
            }
            _ => {
                let mut character = if self.shift_pressed {
                    SHIFT_SCANCODE_SET2[scancode as usize]
                } else {
                    SCANCODE_SET2[scancode as usize]
                };

                // Apply Caps Lock logic
                if character.is_ascii_alphabetic() {
                    if self.caps_lock ^ self.shift_pressed {
                        character = character.to_ascii_uppercase();
                    } else {
                        character = character.to_ascii_lowercase();
                    }
                }

                Some(character)
            }
        }
    }
}

/// Initializes the global keyboard instance.
pub fn init_keyboard() {
    let mut keyboard = KEYBOARD.lock();
    if keyboard.is_none() {
        *keyboard = Some(Keyboard::new());
        if let Some(ref mut kb) = *keyboard {
            kb.init();
        }
    }
}

/// Reads a key from the keyboard, if available.
pub fn read_key() -> Option<char> {
    let mut keyboard = KEYBOARD.lock();
    if let Some(ref mut kb) = *keyboard {
        kb.get_key()
    } else {
        None
    }
}
