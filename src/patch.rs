use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::mem::size_of;

use zerocopy::LayoutVerified;

use anyhow::{anyhow, bail, ensure};

use crate::{EntryHeader, PatchHeader, DDELTA_MAGIC};

use super::Result;
use std::io;

const BLOCK_SIZE: u64 = 32 * 1024;
macro_rules! read {
    ($reader: expr, $type: ty) => {{
        let mut buf = [0; size_of::<$type>()];
        let data: Result<$type> = $reader
            .read_exact(&mut buf)
            .map_err(|err| err.into())
            .and_then(|_| {
                LayoutVerified::<_, $type>::new(&buf[..])
                    .map(|data| *data)
                    .ok_or_else(|| anyhow!("Bytes not aligned"))
            });
        data
    }};
}
fn apply_diff(
    patch_f: &mut impl Read,
    old_f: &mut impl Read,
    new_f: &mut impl Write,
    mut size: u64,
) -> Result<()> {
    let mut old = [0; BLOCK_SIZE as usize];
    let mut patch = [0; BLOCK_SIZE as usize];
    while size > 0 {
        let to_read = BLOCK_SIZE.min(size) as usize;
        let old = &mut old[..to_read];
        let patch = &mut patch[..to_read];

        patch_f.read_exact(patch)?;
        old_f.read_exact(old)?;

        old.iter_mut()
            .zip(patch.iter())
            .for_each(|(old, patch)| *old = old.wrapping_add(*patch));

        new_f.write_all(&old)?;

        size -= to_read as u64;
    }
    Ok(())
}

fn copy_bytes(src: &mut impl Read, dst: &mut impl Write, mut bytes: u64) -> Result<()> {
    let mut buf = [0; BLOCK_SIZE as usize];
    while bytes > 0 {
        let to_read = BLOCK_SIZE.min(bytes) as usize;
        let buf = &mut buf[..to_read];
        src.read_exact(buf)?;
        dst.write_all(buf)?;
        bytes -= to_read as u64;
    }
    Ok(())
}

fn apply_with_header(
    old: &mut (impl Read + Seek),
    new: &mut impl Write,
    patch: &mut impl Read,
    header: PatchHeader,
) -> Result<()> {
    ensure!(&header.magic == DDELTA_MAGIC, "Invalid magic number");
    let mut bytes_written = 0;
    loop {
        let entry = read!(patch, EntryHeader)?;
        if entry.diff.get() == 0 && entry.extra.get() == 0 && entry.seek.get() == 0 {
            return if bytes_written == header.new_file_size.get() {
                Ok(())
            } else {
                bail!("Patch too short");
            };
        }
        apply_diff(patch, old, new, entry.diff.get())?;
        copy_bytes(patch, new, entry.extra.get())?;
        old.seek(SeekFrom::Current(entry.seek.get()))?;
        bytes_written += entry.diff.get() + entry.extra.get();
    }
}

/// Apply a patch file. This is compatible with the formats created by [`generate`][crate::generate]
/// and the original ddelta program.
///
/// However, it is not compatible with the format created by
/// [`generate_chunked`][crate::generate_chunked]. In that case, use [`apply_chunked`].
pub fn apply(
    old: &mut (impl Read + Seek),
    new: &mut impl Write,
    patch: &mut impl Read,
) -> Result<()> {
    let header = read!(patch, PatchHeader)?;
    apply_with_header(old, new, patch, header)
}

/// Apply a patch file. This is compatible with the formats created by
/// [`generate`][crate::generate], [`generate_chunked`][crate::generate_chunked], as well as the
/// original ddelta program.
pub fn apply_chunked(
    old: &mut (impl Read + Seek),
    new: &mut impl Write,
    patch: &mut impl Read,
) -> Result<()> {
    let mut bytes_written = 0;
    loop {
        let header = match read!(patch, PatchHeader) {
            Ok(header) => header,
            Err(e) => {
                return if e
                    .downcast_ref::<io::Error>()
                    .map_or(false, |e| e.kind() == ErrorKind::UnexpectedEof)
                {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        };
        // Each iteration expects to start from the beginning of the old file, so we can take
        // advantage of the fact that the chunks of old & new are always the same, and if they're
        // not, no data is read from the old file
        old.seek(SeekFrom::Start(bytes_written))?;
        bytes_written += header.new_file_size.get();
        apply_with_header(old, new, patch, header)?;
    }
}
