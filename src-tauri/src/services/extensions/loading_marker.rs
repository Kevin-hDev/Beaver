use std::path::{Path, PathBuf};

#[path = "loading_marker_completion.rs"]
mod completion;

const FILE_NAME: &str = "extension-loading.json";
pub(super) const MAX_MARKER_BYTES: usize = 2_048;
use super::loading_journal_format::LoadingJournal;
use super::loading_journal_store as store;
pub(crate) use super::loading_marker_format::LoadingMarker;

#[cfg(test)]
static TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(test)]
pub(super) async fn test_lock() -> tokio::sync::MutexGuard<'static, ()> {
    TEST_LOCK.lock().await
}

#[derive(Clone, Debug)]
pub(crate) enum MarkerRead {
    Missing,
    Valid(LoadingMarker),
    Invalid,
}

#[derive(Clone, Debug)]
pub(crate) enum JournalRead {
    Missing,
    Valid(LoadingJournal),
    Invalid,
}

pub(super) struct PreservedMarker {
    pub(super) state: MarkerRead,
    pub(super) host: Option<LoadingMarker>,
}

pub(super) fn read() -> MarkerRead {
    read_at(&path())
}

pub(super) fn preserve() -> PreservedMarker {
    preserve_at(&path())
}

pub(super) fn start(extension_id: &str, attempts: u8) -> Result<(), String> {
    start_at(&path(), extension_id, attempts)
}

pub(super) fn advance(extension_id: &str, stage: &str) -> Result<(), String> {
    advance_at(&path(), extension_id, stage)
}

pub(super) fn discard() -> Result<(), String> {
    discard_at(&path())
}

pub(super) fn ui_start(extension_id: &str, attempts: u8) -> Result<(), String> {
    ui_start_at(&path(), extension_id, attempts)
}

pub(crate) fn ui_advance(extension_id: &str, stage: &str) -> Result<(), String> {
    ui_advance_at(&path(), extension_id, stage)
}

pub(crate) fn ui_complete(extension_id: &str) -> Result<(), String> {
    ui_complete_at(&path(), extension_id)
}

pub(super) fn ui_clear_if_matches(extension_id: &str) -> Result<(), String> {
    ui_clear_if_matches_at(&path(), extension_id)
}

pub(super) fn ui_clear_if_matches_at(path: &Path, extension_id: &str) -> Result<(), String> {
    super::validation::identifier(extension_id)?;
    match read_journal_at(path) {
        JournalRead::Missing => Ok(()),
        JournalRead::Invalid => Err(marker_error()),
        JournalRead::Valid(journal) => match journal.ui() {
            Some(marker) if marker.extension_id == extension_id => {
                ui_complete_at(path, extension_id)
            }
            Some(_) | None => Ok(()),
        },
    }
}

#[allow(
    dead_code,
    reason = "stable journal operation consumed by the UI loader in UI-P1"
)]
pub(super) fn ui_discard() -> Result<(), String> {
    store::clear_ui(&path(), None)
}

pub(super) fn next_retry_attempt(extension_id: &str) -> Result<u8, String> {
    next_retry_attempt_at(&path(), extension_id)
}

pub(super) fn next_retry_attempt_at(marker_path: &Path, extension_id: &str) -> Result<u8, String> {
    let MarkerRead::Valid(marker) = read_at(marker_path) else {
        return Err(marker_error());
    };
    if marker.extension_id != extension_id || !marker.can_retry() {
        return Err(marker_error());
    }
    marker.attempts.checked_add(1).ok_or_else(marker_error)
}

pub(super) fn complete(
    preserved: PreservedMarker,
    applied_ids: &std::collections::HashSet<String>,
    resolved_recovery: Option<(&str, u8)>,
) -> Result<(), String> {
    completion::complete_at(&path(), preserved, applied_ids, resolved_recovery)
}

#[cfg(test)]
pub(super) fn complete_at(
    marker_path: &Path,
    preserved: PreservedMarker,
    applied_ids: &std::collections::HashSet<String>,
    resolved_recovery: Option<(&str, u8)>,
) -> Result<(), String> {
    completion::complete_at(marker_path, preserved, applied_ids, resolved_recovery)
}

pub(crate) fn read_at(path: &Path) -> MarkerRead {
    match read_journal_at(path) {
        JournalRead::Missing => MarkerRead::Missing,
        JournalRead::Valid(journal) => journal
            .host()
            .cloned()
            .map_or(MarkerRead::Missing, MarkerRead::Valid),
        JournalRead::Invalid => MarkerRead::Invalid,
    }
}

pub(crate) fn read_journal_at(path: &Path) -> JournalRead {
    store::read(path)
}

pub(super) fn preserve_at(path: &Path) -> PreservedMarker {
    let state = read_at(path);
    let host = match &state {
        MarkerRead::Valid(marker) => Some(marker.clone()),
        MarkerRead::Missing | MarkerRead::Invalid => None,
    };
    PreservedMarker { state, host }
}

pub(crate) fn start_at(path: &Path, extension_id: &str, attempts: u8) -> Result<(), String> {
    store::start_host(path, extension_id, attempts)
}

pub(super) fn advance_at(path: &Path, extension_id: &str, stage: &str) -> Result<(), String> {
    store::advance_host(path, extension_id, stage)
}

#[cfg(test)]
pub(super) fn advance_fail_before_replace_at(
    path: &Path,
    extension_id: &str,
    stage: &str,
) -> Result<(), String> {
    store::advance_host_with_failure(path, extension_id, stage)
}

pub(super) fn ui_start_at(path: &Path, extension_id: &str, attempts: u8) -> Result<(), String> {
    store::start_ui(path, extension_id, attempts)
}

pub(super) fn ui_advance_at(path: &Path, extension_id: &str, stage: &str) -> Result<(), String> {
    store::advance_ui(path, extension_id, stage, false)
}

#[cfg(test)]
pub(super) fn ui_advance_fail_before_replace_at(
    path: &Path,
    extension_id: &str,
    stage: &str,
) -> Result<(), String> {
    store::advance_ui(path, extension_id, stage, true)
}

pub(crate) fn ui_complete_at(path: &Path, extension_id: &str) -> Result<(), String> {
    store::clear_ui(path, Some(extension_id))
}

pub(super) fn discard_at(path: &Path) -> Result<(), String> {
    store::discard_host(path)
}

pub(crate) fn discard_invalid_at(path: &Path) -> Result<(), String> {
    store::discard_invalid(path)
}

pub(crate) fn path() -> PathBuf {
    crate::services::paths::data_dir().join(FILE_NAME)
}

pub(super) fn marker_error() -> String {
    super::error_codes::RECOVERY_MARKER_INVALID.to_string()
}
