use once_cell::sync::OnceCell;

#[inline(always)]
pub(crate) fn page_size() -> usize {
    static PAGE_SIZE: OnceCell<usize> = OnceCell::new();
    *PAGE_SIZE.get_or_init(|| {
        // Safety: See https://man7.org/linux/man-pages/man3/sysconf.3.html
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    })
}
