use core::option::Option::Some;
use core::result::Result::Ok;

use crate::apps::pofetch::pofetch;
use crate::driver::keyboard;
use crate::polib::print::clear;
use crate::poprint;

const BUFFER_SIZE: usize = 128;

pub fn shell() {
    loop {
        // Display a styled prompt
        poprint!("[blue][[local@popoen] ~> [reset]");

        // Initialize the keyboard
        keyboard::init_keyboard();

        let mut buffer = [0u8; BUFFER_SIZE];
        let mut index = 0;

        // Start reading keyboard input
        loop {
            if let Some(c) = keyboard::read_key() {
                match c {
                    '\x08' => {
                        // Handle backspace: move cursor back and overwrite the character
                        if index > 0 {
                            index -= 1;
                            poprint!("\x08 \x08");
                        }
                    }
                    '\n' => {
                        // Print newline and break the loop
                        poprint!("\n");
                        break;
                    }
                    _ => {
                        // Print the character to the screen
                        if index < BUFFER_SIZE {
                            buffer[index] = c as u8;
                            index += 1;
                            poprint!("{}", c);
                        }
                    }
                }
            }
        }

        if index > 0 {
            if pcmd(&buffer[..index]) {
                break;
            }
        }

        index = 0;
    }
}

fn pcmd(command: &[u8]) -> bool {
    if let Ok(cmd_str) = core::str::from_utf8(command) {
        match cmd_str {
            "exit" => {
                true // Return true for the "exit" command
            }
            "pofetch" | "neofetch" => {
                pofetch();
                false
            }
            "clear" => {
                clear();
                false
            }
            _ => {
                poprint!("Unknown command: {}\n", cmd_str);
                false
            }
        }
    } else {
        poprint!("Invalid UTF-8 command\n");
        false
    }
}
