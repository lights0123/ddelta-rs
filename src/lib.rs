use byteorder::BigEndian;
use zerocopy::{AsBytes, FromBytes, Unaligned, I64, U64};

use anyhow::Result;
pub use diff::generate;
pub use patch::apply;

const DDELTA_MAGIC: &[u8; 8] = b"DDELTA40";

mod diff;
mod patch;

#[derive(Debug, Copy, Clone, FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct PatchHeader {
    magic: [u8; 8],
    new_file_size: U64<BigEndian>,
}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct EntryHeader {
    diff: U64<BigEndian>,
    extra: U64<BigEndian>,
    seek: I64<BigEndian>,
}
