use core::iter::Iterator;
use core::option::Option::{self, None, Some};

/// On‑disk superblock
#[repr(C)]
pub struct Superblock {
    pub magic: u32,
    pub total_blocks: u32,
    pub total_inodes: u32,
    pub free_block_bitmap_start: u32,
    pub inode_table_start: u32,
    pub data_blocks_start: u32,
}

/// On‑disk inode
#[repr(C)]
pub struct Inode {
    pub mode: u16,             // file type + permissions
    pub size: u32,             // size in bytes
    pub direct_ptrs: [u32; 12],// direct block pointers
    pub indirect_ptr: u32,     // single indirect
    pub is_used: u8,           // 0 = free, 1 = in use
}

/// Directory entry (fixed‑size name + inode index)
#[repr(C)]
pub struct DirEntry {
    pub inode_index: u32,      // 0 = unused, otherwise inode slot+1
    pub name: [u8; 28],        // up to 27 bytes + NUL
    pub _padding: u16,         // pad to 36 bytes
}

/// Block device trait
pub trait BlockDevice {
    fn read_block(&self, block_number: u32, buf: &mut [u8]);
    fn write_block(&mut self, block_number: u32, buf: &[u8]);
}

/// In‑memory “physical” device: 64 chunks × 64 B = 4096 B
pub struct PPDev {
    pub blocks: [[u8; 64]; 64],
}

impl BlockDevice for PPDev {
    fn read_block(&self, block_number: u32, buf: &mut [u8]) {
        // Each 512 B “logical” block = 8 × 64 B chunks
        let idx = block_number as usize;
        let chunks_per_block = 512 / 64;
        if idx * chunks_per_block + chunks_per_block > self.blocks.len() {
            return; // out of bounds
        }

        let mut bi = 0;
        for i in 0..chunks_per_block {
            let chunk = &self.blocks[idx * chunks_per_block + i];
            for &b in chunk.iter() {
                buf[bi] = b;
                bi += 1;
            }
        }
    }

    fn write_block(&mut self, block_number: u32, buf: &[u8]) {
        let start = (block_number as usize) * 512;

        if buf.len() == 512 {
            // fast full‑block copy
            self.blocks
                .iter_mut()
                .flat_map(|c| c.iter_mut())
                .skip(start)
                .take(512)
                .zip(buf.iter())
                .for_each(|(dst, &src)| *dst = src);
        } else {
            // partial write
            for (i, &b) in buf.iter().enumerate() {
                let off = start + i;
                let block_i = off / 64;
                let byte_i = off % 64;
                self.blocks[block_i][byte_i] = b;
            }
        }
    }
}
