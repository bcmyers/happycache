#![cfg(unix)]

mod mincore;
mod mmap;
mod page_size;

use std::fs;
use std::io;
use std::os::unix::prelude::*;
use std::path::Path;

use crate::mmap::Mmap;
use crate::page_size::page_size;

/// Writes a happycache to `out` based on the contents of `dir`
///
/// # Errors
///
/// TODO
pub fn spider(out: &mut impl io::Write, dir: &Path) -> std::io::Result<()> {
    let mut scratch_buffer = Vec::new();

    for entry in dir.read_dir()? {
        let path = entry?.path();
        let metadata = path.metadata()?;

        if metadata.is_dir() {
            spider(out, &path)?;
        } else if metadata.is_file() && metadata.len() > 0 {
            let f = fs::File::open(&path)?;
            let len = usize::try_from(metadata.len())
                .expect("file length does not exceed usize::MAX bytes");
            // Safety: len corresponds to the length of f
            let mmap = unsafe { Mmap::new(&f, len) }?;
            dump_file(out, &path, &mmap, &mut scratch_buffer)?;
        }
    }

    Ok(())
}

fn dump_file(
    out: &mut impl io::Write,
    path: &Path,
    mmap: &Mmap,
    scratch_buffer: &mut Vec<u8>,
) -> io::Result<()> {
    const PAGES_PER_CHUNK: usize = 1024 * 1024;

    // write header
    out.write_all(path.as_os_str().as_bytes())?;
    out.write_all(b"\n")?;

    // write body
    let mut page_current = 0;
    let mut page_last = None;

    loop {
        let bytes = {
            let start = page_current * page_size();
            if start >= mmap.len() {
                break;
            }
            // Safety: mmap.raw() + start remains within the bounds of the mmap
            let ptr = unsafe {
                mmap.raw().offset(
                    isize::try_from(start).expect("file length does not exceed isize::MAX bytes"),
                )
            };
            let len = std::cmp::min(PAGES_PER_CHUNK * page_size(), mmap.len() - start);
            // Safety: ptr and (ptr + len) remain within the bounds of the mmap
            unsafe { std::slice::from_raw_parts_mut(ptr, len) }
        };

        for is_resident in mincore::mincore(bytes, scratch_buffer)? {
            if !is_resident {
                page_current += 1;
                continue;
            }

            // Write the difference to the file, rather than the whole number. This improves gzip's
            // compression ratio
            let diff = (page_current - page_last.unwrap_or(0)).to_string();

            out.write_all(diff.as_bytes())?;
            out.write_all(b"\n")?;

            page_last = Some(page_current);
            page_current += 1;
        }
    }

    Ok(())
}
