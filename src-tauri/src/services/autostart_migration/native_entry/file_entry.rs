use std::io::ErrorKind;
use std::path::PathBuf;

use super::super::{ExactEntryState, ExactLaunchEntry, MigrationError};

const MAX_ENTRY_BYTES: u64 = 16 * 1024;

pub(in crate::services::autostart_migration) struct FileEntry {
    path: PathBuf,
    expected: Vec<u8>,
}

impl FileEntry {
    pub(in crate::services::autostart_migration) fn new(path: PathBuf, expected: Vec<u8>) -> Self {
        Self { path, expected }
    }
}

impl ExactLaunchEntry for FileEntry {
    fn state(&self) -> Result<ExactEntryState, MigrationError> {
        match crate::services::private_store::read_bounded_regular(&self.path, MAX_ENTRY_BYTES)
            .map_err(|_| MigrationError::State)?
        {
            crate::services::private_store::BoundedFile::Missing => Ok(ExactEntryState::Absent),
            crate::services::private_store::BoundedFile::Content(content) => {
                Ok(if content == self.expected {
                    ExactEntryState::Exact
                } else {
                    ExactEntryState::Stale
                })
            }
        }
    }

    fn install(&self) -> Result<(), MigrationError> {
        crate::services::private_store::atomic_write(&self.path, &self.expected)
            .map_err(|_| MigrationError::State)
    }

    fn remove(&self) -> Result<(), MigrationError> {
        remove(&self.path)
    }
}

pub(in crate::services::autostart_migration) fn remove(
    path: &std::path::Path,
) -> Result<(), MigrationError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(_) => Err(MigrationError::State),
    }
}
