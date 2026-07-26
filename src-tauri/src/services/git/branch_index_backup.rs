use std::path::PathBuf;

use git2::Repository;

use super::action_error::GitActionError;

pub(super) struct IndexBackup {
    index_path: PathBuf,
    backup: Option<tempfile::NamedTempFile>,
}

impl IndexBackup {
    pub(super) fn capture(repo: &Repository) -> Result<Self, GitActionError> {
        let index_path = repo.path().join("index");
        if !index_path.exists() {
            return Ok(Self {
                index_path,
                backup: None,
            });
        }
        let backup = tempfile::NamedTempFile::new_in(repo.path())
            .map_err(|_| GitActionError::CommitFailed)?;
        std::fs::copy(&index_path, backup.path()).map_err(|_| GitActionError::CommitFailed)?;
        Ok(Self {
            index_path,
            backup: Some(backup),
        })
    }

    pub(super) fn restore(&self) {
        if let Some(backup) = &self.backup {
            let _ = std::fs::copy(backup.path(), &self.index_path);
        } else {
            let _ = std::fs::remove_file(&self.index_path);
        }
    }
}
