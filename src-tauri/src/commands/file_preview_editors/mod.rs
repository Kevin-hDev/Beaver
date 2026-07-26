use serde::Serialize;
use std::path::Path;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone, Serialize)]
pub struct DetectedEditor {
    pub name: String,
    pub path: String,
    pub is_default: bool,
}

pub fn detect_editors_for_extension(file_path: &Path) -> Vec<DetectedEditor> {
    #[cfg(target_os = "macos")]
    return macos::detect(file_path);

    #[cfg(target_os = "windows")]
    return windows::detect(file_path);

    #[cfg(target_os = "linux")]
    return linux::detect(file_path);

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return vec![];
}
