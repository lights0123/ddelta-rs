use super::Result;
use crate::{EntryHeader, PatchHeader, DDELTA_MAGIC};
use anyhow::{anyhow, bail, ensure};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::size_of;
use zerocopy::LayoutVerified;

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
        src.read_exact(&mut buf)?;
        dst.write_all(&buf)?;
        bytes -= to_read as u64;
    }
    Ok(())
}

pub fn apply(
    patch: &mut impl Read,
    old: &mut (impl Read + Seek),
    new: &mut impl Write,
) -> Result<()> {
    let header = read!(patch, PatchHeader)?;
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
