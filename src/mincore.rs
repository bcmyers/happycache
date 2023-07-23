use std::io;

use crate::page_size::page_size;

// Return an iterator whose length is the number of pages spanned by the provided input slice. Each
// item produced by the iterator is a boolean indicating whether or not the corresponding page is
// resident in memory
//
// # Errors
//
// Returns an error if:
// * the input slice is not page-aligned
// * the call to libc::mincore returns an error
pub(crate) fn mincore<'a>(
    input: &[u8],
    scratch_buffer: &'a mut Vec<u8>,
) -> io::Result<impl Iterator<Item = bool> + 'a> {
    // Check that input is page-aligned
    if (input.as_ptr() as usize) % page_size() != 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "input must be page-aligned",
        ));
    }

    // Clear buffer and resize it to fit expectations of libc::mincore
    scratch_buffer.clear();
    let buffer_len = (input.len() + page_size() - 1) / page_size();
    scratch_buffer.resize(buffer_len, 0u8);

    // Call libc::mincore
    // Safety: See https://man7.org/linux/man-pages/man2/mincore.2.html
    let ret = unsafe {
        libc::mincore(
            /* addr   */ input.as_ptr() as *const _,
            /* length */ input.len(),
            /* vec    */ scratch_buffer.as_mut_ptr() as *mut _,
        )
    };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(scratch_buffer.iter().map(|&byte| {
        // The least significant bit of each byte will be set if the corresponding page is currently
        // resident in memory, and be clear otherwise. (The settings of the other bits in each byte
        // are undefined; these bits are reserved for possible later use.)
        byte & 1 != 0
    }))
}
