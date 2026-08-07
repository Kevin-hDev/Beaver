use super::native_paths::{resolve_runtime_files, RuntimeFiles};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct BrowserLibraryGuard {
    runtime_files: RuntimeFiles,
}

impl BrowserLibraryGuard {
    pub(crate) fn load_for_current_process() -> Result<Self, ()> {
        let executable = std::env::current_exe().map_err(|_| ())?;
        let downloaded = cef::sys::get_cef_dir();
        let files = resolve_runtime_files(&executable, downloaded.as_deref()).ok_or(())?;
        Self::load(&files.framework)?;
        Ok(Self {
            runtime_files: files,
        })
    }

    pub(super) fn runtime_files(&self) -> &RuntimeFiles {
        &self.runtime_files
    }

    fn load(framework: &Path) -> Result<(), ()> {
        let path = CString::new(framework.as_os_str().as_bytes()).map_err(|_| ())?;
        // cef-rs exposes this C string as a reference to its first byte. The
        // CString stays alive for the whole synchronous load_library call.
        let first_byte = unsafe { path.as_ptr().as_ref() }.ok_or(())?;
        if cef::load_library(Some(first_byte)) != 1 {
            return Err(());
        }
        Ok(())
    }
}

impl Drop for BrowserLibraryGuard {
    fn drop(&mut self) {
        let _ = cef::unload_library();
    }
}
