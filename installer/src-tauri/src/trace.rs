use crate::error::InstallerError;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[path = "trace_store.rs"]
mod store;

pub const MAX_TRACE_BYTES: usize = 256 * 1024;
const ACTIVE_TRACE_NAME: &str = ".installer-trace.jsonl";

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationId {
    Download,
    Verify,
    Install,
    Cleanup,
}

impl Drop for InstallerTrace {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    Started,
    Running,
    Finished,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Pending,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ErrorCode {
    #[serde(rename = "installer-invalid-launch")]
    InvalidLaunch,
    #[serde(rename = "installer-download-failed")]
    DownloadFailed,
    #[serde(rename = "installer-integrity-failed")]
    IntegrityFailed,
    #[serde(rename = "installer-cleanup-failed")]
    CleanupFailed,
    #[serde(rename = "installer-install-failed")]
    InstallFailed,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TraceEntry {
    pub operation_id: OperationId,
    pub phase: Phase,
    pub outcome: Outcome,
    pub elapsed_ms: u64,
    pub error_code: Option<ErrorCode>,
}

pub struct InstallerTrace {
    pub(super) path: PathBuf,
    pub(super) file: Option<File>,
    bytes: usize,
}

impl InstallerTrace {
    pub fn create(run: &Path) -> Result<Self, InstallerError> {
        Self::create_in(run)
    }

    pub(crate) fn create_in(run: &Path) -> Result<Self, InstallerError> {
        let path = run.join(ACTIVE_TRACE_NAME);
        let file = store::create_private_file(&path)?;
        Ok(Self {
            path,
            file: Some(file),
            bytes: 0,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn record(&mut self, entry: TraceEntry) -> Result<(), InstallerError> {
        let mut line = serde_json::to_vec(&entry).map_err(|_| InstallerError::CleanupFailed)?;
        line.push(b'\n');
        let next = self
            .bytes
            .checked_add(line.len())
            .filter(|size| *size <= MAX_TRACE_BYTES)
            .ok_or(InstallerError::CleanupFailed)?;
        let file = self.file.as_mut().ok_or(InstallerError::CleanupFailed)?;
        file.write_all(&line)
            .and_then(|_| file.sync_data())
            .map_err(|_| InstallerError::CleanupFailed)?;
        self.bytes = next;
        Ok(())
    }

    pub fn preserve(mut self, run_id: &str) -> Result<PathBuf, InstallerError> {
        store::preserve(&mut self, &beaver_data_path::data_dir(), run_id)
    }

    #[cfg(test)]
    pub(crate) fn preserve_into(
        &mut self,
        data_root: &Path,
        run_id: &str,
    ) -> Result<PathBuf, InstallerError> {
        store::preserve(self, data_root, run_id)
    }
}
