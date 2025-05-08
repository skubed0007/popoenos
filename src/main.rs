#![cfg_attr(not(test), no_std, no_main)]

use core::clone::Clone;
use core::option::Option::{self, None, Some};
use core::panic::PanicInfo;
use core::prelude::v1::derive;
use polib::print::clear;
use spin::Mutex;
use crate::fs::structure::{BlockDevice, Inode, PPDev};
use crate::fs::utils::mkfs;
use driver::shell::shell;
lazy_static::lazy_static!{
    // Replacing UnsafeCell with Mutex for thread-safe access
    pub static ref GLOBAL_DEVICE: Mutex<Option<PPDev>> = Mutex::new(None);
    pub static ref ROOT_INODE: Mutex<Option<Inode>> = Mutex::new(None);
}


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    poprint!("PANIC!");
    loop {}
}

pub mod apps;
pub mod driver;
pub mod fs;
pub mod polib;


pub fn list_files(device: &dyn BlockDevice, inode_table: &[Inode], files: &mut [&str]) {
    let mut index = 0;

    for inode in inode_table {
        if inode.size > 0 {
            if index < files.len() {
                files[index] = "file"; // Placeholder for file name
                index += 1;
            } else {
                break;
            }
        }
    }

    // Fill remaining slots with empty strings
    for i in index..files.len() {
        files[i] = "";
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    pomain();
    loop {}
}
pub fn pomain() {
    poprint!("[pomain] Starting kernel main...\n");

    poprint!("[pomain] Creating device with 1024 blocks...\n");
    let mut device = PPDev { blocks: [[0; 64]; 64] };

    poprint!("[pomain] Formatting device with mkfs...\n");
    mkfs(&mut device);

    poprint!("[pomain] Locking GLOBAL_DEVICE...\n");
    let mut global = GLOBAL_DEVICE.lock();
    poprint!("[pomain] GLOBAL_DEVICE locked. Installing device...\n");
    global.replace(device);

    poprint!("[pomain] Creating root inode...\n");
    let root_inode = Inode {
        mode: 0o755,
        size: 0,
        direct_ptrs: [0; 12],
        indirect_ptr: 0,
        is_used: 1,
    };
    ROOT_INODE.lock().replace(root_inode);
    poprint!("[pomain] Root inode installed.\n");

    poprint!("[pomain] Launching shell...\n");
    core::mem::drop(global);

    shell();
}
