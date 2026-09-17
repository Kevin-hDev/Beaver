//! Versioned ownership journal: the store is its only writer.
use super::super::types::ExtensionRecord;
use super::{InstallPhase, InstallRequest};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(super) const FORMAT: u32 = 1;
pub(super) const MAX_JOURNAL_BYTES: u64 = 4 * 1024 * 1024;
const DIRECTORY: &str = "extension-install-jobs";
const JOURNAL: &str = "jobs.json";

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::services::extensions) struct InstallCheckpoint {
    pub version: u32,
    pub token: String,
    pub resolved_source: Option<InstallRequest>,
    pub safe_phase: Option<InstallPhase>,
    pub record: Option<ExtensionRecord>,
    pub previous: Option<ExtensionRecord>,
    #[serde(default)]
    pub allowance: super::disk_policy::StorageAllowance,
    #[serde(default)]
    pub dependency_lock: Option<super::dependency_lock::DependencyLock>,
    pub producer_active: bool,
    #[serde(default)]
    pub native_process: Option<crate::services::owned_process::OwnedProcessIdentity>,
    pub cleanup_unconfirmed: bool,
}

pub(super) fn path() -> PathBuf {
    crate::services::paths::data_dir()
        .join(DIRECTORY)
        .join(JOURNAL)
}

#[derive(Serialize, Deserialize)]
pub(super) struct Journal {
    pub version: u32,
    pub revision: u64,
    pub jobs: Vec<super::store::Job>,
}

pub(super) fn load(path: &std::path::Path) -> Result<Option<Journal>, String> {
    use crate::services::private_store::{read_bounded_regular, BoundedFile};
    let bytes = match read_bounded_regular(path, MAX_JOURNAL_BYTES)? {
        BoundedFile::Missing => return Ok(None),
        BoundedFile::Content(bytes) => bytes,
    };
    let journal: Journal =
        serde_json::from_slice(&bytes).map_err(|_| super::limits::UNAVAILABLE)?;
    if journal.version != FORMAT
        || journal.revision >= super::limits::MAX_REVISION - 1
        || journal.jobs.len() > super::limits::MAX_ACTIVE + super::limits::MAX_RECENT
        || journal
            .jobs
            .iter()
            .filter(|job| !job.view.status.terminal())
            .count()
            > super::limits::MAX_ACTIVE
        || journal
            .jobs
            .iter()
            .filter(|job| job.view.status.terminal())
            .count()
            > super::limits::MAX_RECENT
    {
        return Err(super::limits::UNAVAILABLE.into());
    }
    for (index, job) in journal.jobs.iter().enumerate() {
        super::request::id(&job.view.id)?;
        if job.view.revision > journal.revision
            || job
                .finished_revision
                .is_some_and(|revision| revision > journal.revision)
            || journal.jobs[..index]
                .iter()
                .any(|other| other.view.id == job.view.id)
        {
            return Err(super::limits::UNAVAILABLE.into());
        }
        if let Some(checkpoint) = &job.checkpoint {
            if let Some(identity) = checkpoint.native_process {
                if identity.pid < 2
                    || identity.pid > i32::MAX as u32
                    || identity.native_start_time == 0
                    || identity.executable == 0
                {
                    return Err(super::limits::UNAVAILABLE.into());
                }
                #[cfg(unix)]
                if identity.native_scope != u64::from(identity.pid) {
                    return Err(super::limits::UNAVAILABLE.into());
                }
            }
            if !valid_token(&checkpoint.token) {
                return Err(super::limits::UNAVAILABLE.into());
            }
        }
    }
    Ok(Some(journal))
}

pub(super) fn valid_token(token: &str) -> bool {
    token.len() == 32
        && token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Startup cleaners must not guess ownership before job reconciliation.
/// A corrupt journal retains artifacts and blocks new work instead of erasing evidence.
pub(crate) fn protects_artifacts() -> bool {
    match load(&path()) {
        Ok(None) => false,
        Ok(Some(journal)) => journal.jobs.iter().any(|job| !job.clean),
        Err(_) => true,
    }
}
