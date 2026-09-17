use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(super) enum Platform {
    #[cfg(any(test, target_os = "macos"))]
    Macos,
    #[cfg(any(test, target_os = "linux"))]
    Linux,
    #[cfg(any(test, windows))]
    Windows,
}

pub(super) fn current() -> Platform {
    #[cfg(target_os = "macos")]
    return Platform::Macos;
    #[cfg(target_os = "linux")]
    return Platform::Linux;
    #[cfg(windows)]
    return Platform::Windows;
}

pub(super) fn default_cache_base(platform: Platform, home: &Path) -> PathBuf {
    match platform {
        #[cfg(any(test, target_os = "macos"))]
        Platform::Macos => home.join("Library/Caches"),
        #[cfg(any(test, target_os = "linux"))]
        Platform::Linux => home.join(".cache"),
        #[cfg(any(test, windows))]
        Platform::Windows => home.join("AppData/Local"),
    }
}

pub(super) fn is_windows(platform: Platform) -> bool {
    #[cfg(any(test, windows))]
    if matches!(platform, Platform::Windows) {
        return true;
    }
    let _ = platform;
    false
}
