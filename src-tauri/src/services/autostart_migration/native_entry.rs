use std::path::{Path, PathBuf};

use super::{ExactEntryState, ExactLaunchEntry, MigrationError};

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
pub(super) mod documents;
#[cfg(any(test, target_os = "linux", target_os = "macos"))]
pub(super) mod file_entry;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform;
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform;
#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

pub(super) struct NativeEntry {
    name: String,
    executable: PathBuf,
}

impl NativeEntry {
    pub(super) fn new(name: &str, executable: &Path) -> Result<Self, MigrationError> {
        let valid_name = !name.is_empty()
            && name.len() <= 128
            && name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-');
        if !valid_name || !executable.is_absolute() || !executable.is_file() {
            return Err(MigrationError::Setup);
        }
        Ok(Self {
            name: name.to_string(),
            executable: executable.to_path_buf(),
        })
    }
}

impl ExactLaunchEntry for NativeEntry {
    fn state(&self) -> Result<ExactEntryState, MigrationError> {
        platform::state(&self.name, &self.executable)
    }

    fn install(&self) -> Result<(), MigrationError> {
        platform::install(&self.name, &self.executable)
    }

    fn remove(&self) -> Result<(), MigrationError> {
        platform::remove(&self.name)
    }
}
