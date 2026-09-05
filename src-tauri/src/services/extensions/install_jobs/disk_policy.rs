//! Capacity probe shared with the forthcoming consent policy.
use std::path::Path;
pub(super) const PUBLICATION_RESERVE: u64 = 8 * 1024 * 1024;

#[allow(clippy::unnecessary_cast)] // statvfs field widths differ across supported Unix targets.
pub(super) fn free_bytes(path: &Path) -> Result<u64, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|_| super::limits::INVALID)?;
        let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: CString is terminated and stat points to writable storage.
        if unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
            return Err(super::limits::UNAVAILABLE.into());
        }
        let stat = unsafe { stat.assume_init() };
        (stat.f_bavail as u64)
            .checked_mul(stat.f_frsize as u64)
            .ok_or(super::limits::UNAVAILABLE.into())
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut available = 0_u64;
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                path.as_ptr(),
                &mut available,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(super::limits::UNAVAILABLE.into())
        } else {
            Ok(available)
        }
    }
}
