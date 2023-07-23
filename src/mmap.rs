use std::fs;
use std::io;
use std::os::fd::AsRawFd as _;
use std::ptr::NonNull;

pub(crate) struct Mmap {
    ptr: NonNull<u8>,
    len: usize,
}

impl Mmap {
    /// Safety: The provided length must correspond to the length of the file
    pub(crate) unsafe fn new(file: &fs::File, len: usize) -> io::Result<Mmap> {
        let fd = file.as_raw_fd();

        // Safety: See https://man7.org/linux/man-pages/man2/mmap.2.html
        let raw = unsafe {
            libc::mmap(
                /* addr   */ std::ptr::null_mut(),
                /* length */ len,
                /* prot   */ libc::PROT_NONE,
                /* flags  */ libc::MAP_PRIVATE,
                /* fd     */ fd,
                /* offset */ 0,
            )
        };

        let ptr = NonNull::new(raw as *mut _).ok_or_else(io::Error::last_os_error)?;

        Ok(Self { ptr, len })
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn raw(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        // Any errors during unmapping/closing are ignored as the only way
        // to report them would be through panicking which is highly discouraged
        // in Drop impls, c.f. https://github.com/rust-lang/lang-team/issues/97
        //
        // Safety: See https://man7.org/linux/man-pages/man3/munmap.3p.html
        let _ = unsafe { libc::munmap(self.raw() as *mut _, self.len as libc::size_t) };
    }
}
