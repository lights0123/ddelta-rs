//! A rust port of [ddelta], which is a streaming and more efficient version of [bsdiff]. The
//! output created by this program is sometimes (when using [`generate`]) compatible with the
//! original C tool, [ddelta], but not with [bsdiff]. This library may use up to 5 times the old
//! file size + the new file size, (5 × min(o, 2^31-1) + min(n, 2^31-1)), up to 12GiB. To control
//! this, see the `chunk_sizes` parameter of [`generate_chunked`].
//!
//! **Note**: the patches created by program should be compressed. If not compressed, the output may
//! actually be larger than just including the new file. You might want to feed the patch file
//! directly to an [encoder][XzEncoder], and read via a
//! [decoder implementing a compression algorithm][XzDecoder] to not require much disk space.
//! Additionally, no checksum is performed, so you should strongly consider doing a checksum of at
//! least either the old or new file once written.
//!
//! ## Features
//!
//! This crate optionally supports compiling the c library, divsufsort, which is enabled by default.
//! A Rust port is available; however, it has worse performance than the C version. If you'd like
//! to use the Rust version instead, for example if you don't have a C compiler installed, add
//! `default-features = false` to your Cargo.toml, i.e.
//!
//! ```toml
//! [dependencies]
//! ddelta = { version = "0.1.0", default-features = false }
//! ```
//!
//! [ddelta]: https://github.com/julian-klode/ddelta
//! [bsdiff]: http://www.daemonology.net/bsdiff/
//! [XzEncoder]: https://docs.rs/xz2/*/xz2/write/struct.XzEncoder.html
//! [XzDecoder]: https://docs.rs/xz2/*/xz2/read/struct.XzDecoder.html

use byteorder::BigEndian;
use zerocopy::{AsBytes, FromBytes, Unaligned, I64, U64};

use anyhow::Result;
pub use diff::{generate, generate_chunked};
pub use patch::{apply, apply_chunked};

const DDELTA_MAGIC: &[u8; 8] = b"DDELTA40";

mod diff;
mod patch;

/// The current state of the generator.
///
/// Passed to a callback periodically to give feedback, such as updating a progress bar.
#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum State {
    /// The new or old file is currently being read. This is currently only used in
    /// [`generate_chunked`].
    Reading,
    /// The internal algorithm, divsufsort, is currently being run.
    Sorting,
    /// The generator is currently working its way through the data. The number represents how much
    /// of the new file has been worked through. In other words, if calculating a percentage, divide
    /// this number by the size of the new file.
    Working(u64),
}

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
