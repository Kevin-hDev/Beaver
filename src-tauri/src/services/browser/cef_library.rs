use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub(super) struct CefLibrary;

impl CefLibrary {
    pub(super) fn load(framework: &Path) -> Result<Self, ()> {
        let path = CString::new(framework.as_os_str().as_bytes()).map_err(|_| ())?;
        // cef-rs exposes this C string as a reference to its first byte. The
        // CString stays alive for the whole synchronous load_library call.
        let first_byte = unsafe { path.as_ptr().as_ref() }.ok_or(())?;
        if cef::load_library(Some(first_byte)) != 1 {
            return Err(());
        }
        Ok(Self)
    }
}

impl Drop for CefLibrary {
    fn drop(&mut self) {
        let _ = cef::unload_library();
    }
}
