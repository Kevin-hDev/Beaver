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
    let expected = documents::macos_launch_agent(name, executable)?;
    Ok(FileEntry::new(entry_path(name)?, expected))
}

fn entry_path(name: &str) -> Result<PathBuf, MigrationError> {
    let home = dirs::home_dir().ok_or(MigrationError::Setup)?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{name}.plist")))
}
