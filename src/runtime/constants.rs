pub static MAGIC_NUMBER: [u8; 4] = [00, 0x61, 0x73, 0x6d];
pub static VERSION: [u8; 4] = [01, 0x00, 0x00, 0x00];

pub static MAX_NUMBER_OF_BYTE_U32: u32 = 5; // ceil ( 32 / 7 )
pub static MAX_NUMBER_OF_BYTE_U64: u32 = 10; // ceil ( 64 / 7 )
pub const CALLSTACK_SIZE: usize = 4 * 1024;
pub const STACK_SIZE: usize = 4 * 1024;

pub const MAX_BR_TABLE: usize = 4 * 1024;

pub const PAGE_SIZE: usize = 64 * 1024;
