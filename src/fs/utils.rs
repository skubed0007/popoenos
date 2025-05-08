use core::result::Result::Ok;
pub use core::{
    cmp::Ord,
    iter::Iterator,
    option::Option::{self, None, Some},
};

use crate::{fs::structure::{Inode, Superblock}, poprint, PPDev};

use super::structure::{BlockDevice, DirEntry};

pub fn mkfs(device: &mut dyn BlockDevice) {
    poprint!("[mkfs] Entered mkfs()\n");

    let sb = Superblock {
        magic: 0x50504f53, // "PPOS"
        total_blocks: 1024,
        total_inodes: 128,
        free_block_bitmap_start: 1,
        inode_table_start: 2,
        data_blocks_start: 10,
    };

    poprint!("[mkfs] Superblock constructed\n");

    // Create a zeroed buffer
    let mut buf = [0u8; 512];
    poprint!("[mkfs] Zeroed buffer created\n");
    // After zeroing inode block (block 0) and root‐inode in block 1:
    let dir_block = [0u8; 512];
    device.write_block(2, &dir_block);

    // Convert Superblock to bytes
    let sb_ptr = (&sb as *const Superblock) as *const u8;
    let sb_size = core::mem::size_of::<Superblock>();
    poprint!(
        "[mkfs] Superblock ptr: {:#x}, size: {}\n",
        sb_ptr as usize,
        sb_size
    );

    let sb_bytes = unsafe { core::slice::from_raw_parts(sb_ptr, sb_size) };
    poprint!("[mkfs] Created byte slice from superblock\n");

    // Copy bytes into the buffer
    buf[..sb_bytes.len()].copy_from_slice(sb_bytes);
    poprint!("[mkfs] Copied superblock into buffer\n");

    // Write the block to disk
    poprint!("[mkfs] About to write block 0\n");
    device.write_block(0, &buf);
    poprint!("[mkfs] Block 0 written!\n");
}

pub fn read_file(device: &dyn BlockDevice, inode: &Inode, buf: &mut [u8]) {
    let mut offset = 0;
    for &block in &inode.direct_ptrs {
        if block == 0 || offset >= buf.len() {
            break;
        }
        let mut tmp = [0u8; 512];
        device.read_block(block, &mut tmp);
        let len = (buf.len() - offset).min(512);
        buf[offset..offset + len].copy_from_slice(&tmp[..len]);
        offset += len;
    }
}

pub fn alloc_block(bitmap: &mut [u8]) -> Option<u32> {
    for (i, byte) in bitmap.iter_mut().enumerate() {
        if *byte != 0xFF {
            for bit in 0..8 {
                if (*byte & (1 << bit)) == 0 {
                    *byte |= 1 << bit;
                    return Some((i * 8 + bit) as u32);
                }
            }
        }
    }
    None
}

pub fn list_dir(device: &dyn BlockDevice, dir_inode: &Inode) {
    for &block in &dir_inode.direct_ptrs {
        if block == 0 {
            continue; // skip unused blocks
        }

        let mut buf = [0u8; 512];
        device.read_block(block, &mut buf);

        let entry_count = 512 / core::mem::size_of::<DirEntry>();
        let entries =
            unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const DirEntry, entry_count) };

        for entry in entries {
            if entry.inode_index == 0 {
                continue; // empty entry
            }

            // Print the name manually (no Vec, no String, just char output)
            let mut i = 0;
            while i < entry.name.len() {
                let byte = entry.name[i];
                if byte == 0 {
                    break;
                }
                poprint!("{}", byte as char); // assuming you have a working `print!`
                i += 1;
            }
            poprint!("\n"); // newline after each entry
        }
    }
}
pub fn ls(device: &dyn BlockDevice) {
    // Read the directory block (we reserve block 2 for DirEntry[])
    let mut dir_buf = [0u8; 512];
    device.read_block(2, &mut dir_buf);

    let entry_size = core::mem::size_of::<DirEntry>();
    let max_entries = 512 / entry_size;

    // Header
    poprint!("{:<6}  {}\n", "Type", "Name");
    poprint!("{:-<6}  {:-<28}\n", "", "");

    // For each entry, if in use, look up its inode to get mode
    for i in 0..max_entries {
        let base = i * entry_size;
        let entry: &DirEntry = unsafe {
            &*(dir_buf[base..].as_ptr() as *const DirEntry)
        };

        if entry.inode_index == 0 {
            continue;
        }

        // Extract name (NUL‐terminated)
        let name_len = entry.name.iter().position(|&b| b == 0).unwrap_or(28);
        let name = str::from_utf8(&entry.name[..name_len]).unwrap_or("<invalid>");

        // Read the corresponding inode to check directory bit
        let mut inode_buf = [0u8; 512];
        device.read_block(0, &mut inode_buf);
        let inode_offset = (entry.inode_index as usize - 1) * core::mem::size_of::<Inode>();
        let inode: &Inode = unsafe {
            &*(inode_buf[inode_offset..].as_ptr() as *const Inode)
        };

        // Determine type
        // On Unix, directory bit is 0o40000; here we check the high bit of mode:
        let is_dir = (inode.mode & 0o40000) != 0;
        let kind = if is_dir { "dir" } else { "file" };

        // Print
        poprint!("{:<6}  {}\n", kind, name);
    }
}