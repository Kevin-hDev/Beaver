use super::native_paths::RuntimeFiles;
use super::{cef_preflight::CefPreflightError, cef_unavailable::CefUnavailableCategory};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct BrowserLibraryGuard {
    runtime_files: RuntimeFiles,
    unload_on_drop: AtomicBool,
}

impl BrowserLibraryGuard {
    pub(crate) fn load_for_current_process_with_retry() -> Result<Self, ()> {
        super::cef_preflight::run_with_retry(Self::load_for_current_process, std::thread::sleep)
            .map_err(|error| {
                ::log::warn!(
                    "[browser] preflight unavailable ({})",
                    error.category().code()
                );
            })
    }

    fn load_for_current_process() -> Result<Self, CefPreflightError> {
        let executable = std::env::current_exe()
            .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Object, &error))?;
        let downloaded = cef::sys::get_cef_dir();
        let files = super::native_paths_macos_preflight::resolve_runtime_files(
            &executable,
            downloaded.as_deref(),
        )?;
        Self::load(&files.framework)?;
        Ok(Self {
            runtime_files: files,
            unload_on_drop: AtomicBool::new(true),
        })
    }

    pub(super) fn runtime_files(&self) -> &RuntimeFiles {
        &self.runtime_files
    }

    pub(super) fn suppress_unload_after_failed_initialize(&self) {
        self.unload_on_drop.store(false, Ordering::Release);
    }

    fn load(framework: &Path) -> Result<(), CefPreflightError> {
        let invalid = || CefPreflightError::deterministic(CefUnavailableCategory::Sandbox);
        let path = CString::new(framework.as_os_str().as_bytes()).map_err(|_| invalid())?;
        // cef-rs exposes this C string as a reference to its first byte. The
        // CString stays alive for the whole synchronous load_library call.
        let first_byte = unsafe { path.as_ptr().as_ref() }.ok_or_else(invalid)?;
        if cef::load_library(Some(first_byte)) != 1 {
            return Err(invalid());
        }
        Ok(())
    }
}

impl Drop for BrowserLibraryGuard {
    fn drop(&mut self) {
        if self.unload_on_drop.load(Ordering::Acquire) {
            let _ = cef::unload_library();
        }
    }
}
