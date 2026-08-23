use std::path::{Path, PathBuf};

use super::super::{ExactEntryState, ExactLaunchEntry, MigrationError};
use super::documents;
use super::file_entry::FileEntry;

pub(super) fn state(name: &str, executable: &Path) -> Result<ExactEntryState, MigrationError> {
    entry(name, executable)?.state()
}

pub(super) fn install(name: &str, executable: &Path) -> Result<(), MigrationError> {
    entry(name, executable)?.install()
}

pub(super) fn remove(name: &str) -> Result<(), MigrationError> {
    super::file_entry::remove(&entry_path(name)?)
}

fn entry(name: &str, executable: &Path) -> Result<FileEntry, MigrationError> {
    let expected = documents::linux_desktop(name, executable)?;
    Ok(FileEntry::new(entry_path(name)?, expected))
}

fn entry_path(name: &str) -> Result<PathBuf, MigrationError> {
    Ok(config_home()?
        .join("autostart")
        .join(format!("{name}.desktop")))
}

fn config_home() -> Result<PathBuf, MigrationError> {
    if let Some(value) = std::env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(value);
        return path
            .is_absolute()
            .then_some(path)
            .ok_or(MigrationError::Setup);
    }
    dirs::home_dir()
        .map(|home| home.join(".config"))
        .ok_or(MigrationError::Setup)
}
