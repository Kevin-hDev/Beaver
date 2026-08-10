#[cfg(windows)]
pub(super) fn terminate_process(code: i32) -> ! {
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};

    // SAFETY: pseudo-handle du processus courant, valable sans fermeture explicite.
    let terminated = unsafe { TerminateProcess(GetCurrentProcess(), code as u32) };
    if terminated == 0 {
        std::process::abort();
    }
    std::process::abort()
}

#[cfg(unix)]
pub(super) fn terminate_process(code: i32) -> ! {
    // SAFETY: `_exit` termine immédiatement le processus sans exécuter de destructeur.
    unsafe { libc::_exit(code) }
}

#[cfg(not(any(unix, windows)))]
compile_error!("unsupported shutdown platform");
