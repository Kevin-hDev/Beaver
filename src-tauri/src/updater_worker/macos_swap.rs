use rand::RngCore;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::command::{run_status, CommandSpec};
use super::macos_bundle::{validate_beaver_stage, BundleKind, ValidatedBundle};
use super::WorkerError;

const COPY_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const MAX_PATH_ATTEMPTS: usize = 8;

pub(crate) struct StagedBundle {
    path: PathBuf,
    armed: bool,
}

impl StagedBundle {
    pub(crate) fn copy(
        source: &ValidatedBundle,
        installation_root: &Path,
    ) -> Result<Self, WorkerError> {
        let path = unique_sibling(installation_root, ".Beaver.app.update")?;
        let stage = Self { path, armed: true };
        run_status(&ditto_spec(&source.root, &stage.path), COPY_TIMEOUT)?;
        if validate_beaver_stage(&stage.path).is_err() {
            return Err(WorkerError);
        }
        Ok(stage)
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for StagedBundle {
    fn drop(&mut self) {
        if self.armed {
            let _ = remove_bundle(&self.path);
        }
    }
}

pub(crate) struct InstallTransaction {
    kind: BundleKind,
    previous: PathBuf,
    target: PathBuf,
    backup: Option<PathBuf>,
    active: bool,
}

impl InstallTransaction {
    pub(crate) fn begin(
        current: &ValidatedBundle,
        mut stage: StagedBundle,
    ) -> Result<Self, WorkerError> {
        let parent = current.root.parent().ok_or(WorkerError)?;
        let target = parent.join("Beaver.app");
        let backup = match current.kind {
            BundleKind::Legacy => {
                match std::fs::symlink_metadata(&target) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    _ => return Err(WorkerError),
                }
                None
            }
            BundleKind::Beaver => {
                let backup = unique_sibling(parent, ".Beaver.app.backup")?;
                std::fs::rename(&current.root, &backup).map_err(|_| WorkerError)?;
                Some(backup)
            }
        };
        if std::fs::rename(&stage.path, &target).is_err() {
            if let Some(backup) = &backup {
                let _ = std::fs::rename(backup, &current.root);
            }
            return Err(WorkerError);
        }
        stage.disarm();
        Ok(Self {
            kind: current.kind,
            previous: current.root.clone(),
            target,
            backup,
            active: true,
        })
    }

    pub(crate) fn installed_bundle(&self) -> &Path {
        &self.target
    }

    pub(crate) fn previous_bundle(&self) -> &Path {
        &self.previous
    }

    pub(crate) fn rollback(&mut self) -> Result<(), WorkerError> {
        if !self.active {
            return Ok(());
        }
        let parent = self.target.parent().ok_or(WorkerError)?;
        let failed = unique_sibling(parent, ".Beaver.app.failed")?;
        if std::fs::rename(&self.target, &failed).is_err() {
            self.active = false;
            return Err(WorkerError);
        }
        if let Some(backup) = &self.backup {
            if std::fs::rename(backup, &self.previous).is_err() {
                let _ = std::fs::rename(&failed, &self.target);
                self.active = false;
                return Err(WorkerError);
            }
        }
        self.active = false;
        remove_bundle(&failed)
    }

    pub(crate) fn commit(mut self) -> Result<(), WorkerError> {
        self.active = false;
        match self.kind {
            BundleKind::Legacy => {
                let parent = self.previous.parent().ok_or(WorkerError)?;
                let removal = unique_sibling(parent, ".cl-go-dash-legacy-remove")?;
                std::fs::rename(&self.previous, &removal).map_err(|_| WorkerError)?;
                remove_bundle(&removal)
            }
            BundleKind::Beaver => {
                let backup = self.backup.take().ok_or(WorkerError)?;
                remove_bundle(&backup)
            }
        }
    }

    pub(crate) fn abandon(mut self) {
        self.active = false;
    }
}

impl Drop for InstallTransaction {
    fn drop(&mut self) {
        let _ = self.rollback();
    }
}

pub(crate) fn ditto_spec(source: &Path, destination: &Path) -> CommandSpec {
    CommandSpec::new(
        "/usr/bin/ditto",
        vec![
            source.as_os_str().to_owned(),
            destination.as_os_str().to_owned(),
        ],
    )
}

fn unique_sibling(parent: &Path, marker: &str) -> Result<PathBuf, WorkerError> {
    for _ in 0..MAX_PATH_ATTEMPTS {
        let mut random = [0_u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut random);
        let path = parent.join(format!("{marker}-{}", hex::encode(random)));
        match std::fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(path),
            Err(_) => return Err(WorkerError),
            Ok(_) => continue,
        }
    }
    Err(WorkerError)
}

fn remove_bundle(path: &Path) -> Result<(), WorkerError> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(WorkerError),
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            std::fs::remove_dir_all(path).map_err(|_| WorkerError)
        }
        Ok(_) => Err(WorkerError),
    }
}

#[cfg(test)]
#[path = "macos_swap_tests.rs"]
mod tests;
