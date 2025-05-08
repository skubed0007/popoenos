use super::structure::{BlockDevice, Superblock};

fn mkfs(device: &mut dyn BlockDevice) {
    let sb = Superblock {
        magic: 0x50504f53, // "PPOS"
        total_blocks: 1024,
        total_inodes: 128,
        free_block_bitmap_start: 1,
        inode_table_start: 2,
        data_blocks_start: 10,
    };

    let mut buf = [0u8; 512];
    let sb_bytes = unsafe {
        core::slice::from_raw_parts((&sb as *const Superblock) as *const u8, core::mem::size_of::<Superblock>())
    };
    buf[..sb_bytes.len()].copy_from_slice(sb_bytes);
    device.write_block(0, &buf);
}
