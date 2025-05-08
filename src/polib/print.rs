use core::fmt::{self, write as fmt_write};
use core::iter::Iterator;
use core::ptr;
use core::option::Option::Some;
use core::cmp::Ord;
use core::result::Result::Ok;

use crate::poprint;

const VGA: *mut u8 = 0xb8000 as *mut u8;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;
static mut COL: usize = 0;
static mut ROW: usize = 0;

fn set_cursor(row: usize, col: usize) {
    let pos = row * WIDTH + col;
    
    outb(0x3D4, 0x0F);
    outb(0x3D5, (pos & 0xFF) as u8);
    outb(0x3D4, 0x0E);
    outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
}

fn outb(port: u16, val: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") val);
    }
}

fn scroll_up() {
    unsafe {
        ptr::copy(
            VGA.add(WIDTH * 2),
            VGA,
            WIDTH * (HEIGHT - 1) * 2,
        );
        for i in 0..WIDTH {
            let offset = (HEIGHT - 1) * WIDTH * 2 + i * 2;
            ptr::write_volatile(VGA.add(offset), b' ');
            ptr::write_volatile(VGA.add(offset + 1), 0x07);
        }
    }
}

fn parse_color(n: &[u8]) -> u8 {
    match n {
        b"black"      => 0,
        b"blue"       => 1,
        b"green"      => 2,
        b"cyan"       => 3,
        b"red"        => 4,
        b"magenta"    => 5,
        b"brown"      => 6,
        b"gray"       => 7,
        b"darkgray"   => 8,
        b"lightblue"  => 9,
        b"lightgreen" => 10,
        b"lightcyan"  => 11,
        b"lightred"   => 12,
        b"pink"       => 13,
        b"yellow"     => 14,
        b"white"      => 15,
        _             => 0xFF,
    }
}

fn reset_colors() -> (u8, u8) {
    (0x07, 0x00) // Default VGA colors: white on black
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut buf = [0u8; 1024];
    let mut cursor = 0;
    struct SliceWriter<'a> {
        buf: &'a mut [u8],
        cur: &'a mut usize,
    }
    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            let len = bytes.len().min(self.buf.len() - *self.cur);
            self.buf[*self.cur..*self.cur + len].copy_from_slice(&bytes[..len]);
            *self.cur += len;
            Ok(())
        }
    }
    let _ = fmt_write(&mut SliceWriter { buf: &mut buf, cur: &mut cursor }, args);
    let slice = &buf[..cursor];

    let (mut fg, mut bg) = reset_colors(); // Initialize colors to default
    let mut i = 0;
    unsafe {
        while i < slice.len() {
            match slice[i] {
                b'\n' => {
                    COL = 0;
                    ROW += 1;
                    if ROW >= HEIGHT {
                        scroll_up();
                        ROW = HEIGHT - 1;
                    }
                    i += 1;
                }
                b'\r' => {
                    COL = 0;
                    i += 1;
                }
                b'\t' => {
                    let tab_size = 4;
                    let spaces = tab_size - (COL % tab_size);
                    for _ in 0..spaces {
                        let offset = (ROW * WIDTH + COL) * 2;
                        ptr::write_volatile(VGA.add(offset), b' ');
                        ptr::write_volatile(VGA.add(offset + 1), (bg << 4) | (fg & 0x0F));
                        COL += 1;
                        if COL >= WIDTH {
                            COL = 0;
                            ROW += 1;
                            if ROW >= HEIGHT {
                                scroll_up();
                                ROW = HEIGHT - 1;
                            }
                        }
                    }
                    i += 1;
                }
                b'\x08' | b'\x7F' => {  // Backspace or Delete
                    if COL > 0 {
                        COL -= 1;
                        let offset = (ROW * WIDTH + COL) * 2;
                        ptr::write_volatile(VGA.add(offset), b' ');
                        ptr::write_volatile(VGA.add(offset + 1), (bg << 4) | (fg & 0x0F));
                    }
                    i += 1;
                }
                b'[' => {
                    if i + 1 < slice.len() && slice[i + 1] == b'[' {
                        // Escaped '[' sequence
                        let offset = (ROW * WIDTH + COL) * 2;
                        ptr::write_volatile(VGA.add(offset), b'[');
                        ptr::write_volatile(VGA.add(offset + 1), (bg << 4) | (fg & 0x0F));
                        COL += 1;
                        i += 2;
                        continue;
                    }
                    if let Some(end) = slice[i..].iter().position(|&c| c == b']') {
                        if i + end + 1 < slice.len() && slice[i + end + 1] == b']' {
                            // Escaped ']' sequence
                            let offset = (ROW * WIDTH + COL) * 2;
                            ptr::write_volatile(VGA.add(offset), b']');
                            ptr::write_volatile(VGA.add(offset + 1), (bg << 4) | (fg & 0x0F));
                            COL += 1;
                            i += end + 2;
                            continue;
                        }
                        let tag = &slice[i + 1..i + end];
                        let mut parts = tag.split(|&c| c == b' ');
                        let first = parts.next();
                        let second = parts.next();
                        let mut valid = true;

                        if let Some(f) = first {
                            if f.starts_with(b"bg") {
                                let code = parse_color(&f[2..]);
                                if code != 0xFF { bg = code } else { valid = false }
                            } else if f == b"reset" {
                                let (default_fg, default_bg) = reset_colors();
                                fg = default_fg;
                                bg = default_bg;
                            } else {
                                let code = parse_color(f);
                                if code != 0xFF { fg = code } else { valid = false }
                            }
                        }

                        if let Some(f2) = second {
                            let code = parse_color(f2);
                            if code != 0xFF { fg = code } else { valid = false }
                        }

                        i += end + 1;
                        if !valid {
                            for _ in 0..2 {
                                let offset = (ROW * WIDTH + COL) * 2;
                                ptr::write_volatile(VGA.add(offset), b'?');
                                ptr::write_volatile(VGA.add(offset + 1), 0x04);
                                COL += 1;
                                if COL >= WIDTH {
                                    COL = 0;
                                    ROW += 1;
                                    if ROW >= HEIGHT {
                                        scroll_up();
                                        ROW = HEIGHT - 1;
                                    }
                                }
                            }
                        }
                        continue;
                    }
                    i += 1;
                }
                c => {
                    let offset = (ROW * WIDTH + COL) * 2;
                    ptr::write_volatile(VGA.add(offset), c);
                    ptr::write_volatile(VGA.add(offset + 1), (bg << 4) | (fg & 0x0F));
                    COL += 1;
                    if COL >= WIDTH {
                        COL = 0;
                        ROW += 1;
                        if ROW >= HEIGHT {
                            scroll_up();
                            ROW = HEIGHT - 1;
                        }
                    }
                    i += 1;
                }
            }
            set_cursor(ROW, COL);
        }
    }
}
pub fn clear() {
    unsafe {
        let default_attr = 0x07; // White on black
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                let offset = (row * WIDTH + col) * 2;
                ptr::write_volatile(VGA.add(offset), b' ');
                ptr::write_volatile(VGA.add(offset + 1), default_attr);
            }
        }
        ROW = 0;
        COL = 0;
        set_cursor(ROW, COL);
    }
}
