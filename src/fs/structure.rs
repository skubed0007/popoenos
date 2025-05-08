#[repr(C)]
pub struct Superblock {
    pub magic: u32,           // Magic number to identify FS type
    pub total_blocks: u32,
    pub total_inodes: u32,
    pub  free_block_bitmap_start: u32,
    pub inode_table_start: u32,
    pub data_blocks_start: u32,
}
#[repr(C)]
pub struct Inode {
    pub mode: u16,         // File type + permissions
    pub size: u32,         // File size in bytes
    pub direct_ptrs: [u32; 12], // Points directly to data blocks
    pub indirect_ptr: u32,      // For large files
    pub is_used: u8,
}
pub trait BlockDevice {
    fn read_block(&self, block_number: u32, buf: &mut [u8]);
    fn write_block(&mut self, block_number: u32, buf: &[u8]);
}
