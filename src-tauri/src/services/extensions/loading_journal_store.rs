use std::path::Path;
use std::sync::Mutex;

use super::loading_journal_format::LoadingJournal;
use super::loading_marker::{marker_error, JournalRead, MAX_MARKER_BYTES};
use super::loading_marker_format::LoadingMarker;

static MUTATION_LOCK: Mutex<()> = Mutex::new(());

pub(super) fn read(path: &Path) -> JournalRead {
    let bytes =
        match crate::services::private_store::read_bounded_regular(path, MAX_MARKER_BYTES as u64) {
            Ok(crate::services::private_store::BoundedFile::Missing) => {
                return JournalRead::Missing
            }
            Ok(crate::services::private_store::BoundedFile::Content(bytes)) => bytes,
            Err(_) => return JournalRead::Invalid,
        };
    super::loading_journal_format::parse(&bytes).map_or(JournalRead::Invalid, JournalRead::Valid)
}

pub(super) fn start_host(path: &Path, extension_id: &str, attempts: u8) -> Result<(), String> {
    let marker = LoadingMarker::new_host(extension_id, attempts)?;
    update(path, false, |journal| {
        *journal.host_mut() = Some(marker);
        Ok(())
    })
}

pub(super) fn start_ui(path: &Path, extension_id: &str, attempts: u8) -> Result<(), String> {
    let marker = LoadingMarker::new_ui(extension_id, attempts)?;
    update(path, false, |journal| {
        *journal.ui_mut() = Some(marker);
        Ok(())
    })
}

pub(super) fn advance_host(path: &Path, extension_id: &str, stage: &str) -> Result<(), String> {
    advance(path, extension_id, stage, false, false)
}

#[cfg(test)]
pub(super) fn advance_host_with_failure(
    path: &Path,
    extension_id: &str,
    stage: &str,
) -> Result<(), String> {
    advance(path, extension_id, stage, false, true)
}

pub(super) fn advance_ui(
    path: &Path,
    extension_id: &str,
    stage: &str,
    fail_before_replace: bool,
) -> Result<(), String> {
    advance(path, extension_id, stage, true, fail_before_replace)
}

fn advance(
    path: &Path,
    extension_id: &str,
    stage: &str,
    ui: bool,
    fail_before_replace: bool,
) -> Result<(), String> {
    let stages = if ui {
        super::ui_contract::UI_LOADING_STAGES
    } else {
        super::types::HOST_LOAD_STAGES
    };
    if !stages.contains(&stage) {
        return Err(marker_error());
    }
    update(path, fail_before_replace, |journal| {
        let entry = if ui {
            journal.ui_mut()
        } else {
            journal.host_mut()
        };
        let Some(entry) = entry.as_mut() else {
            return Err(marker_error());
        };
        if entry.extension_id != extension_id {
            return Err(marker_error());
        }
        entry.stage = stage.to_string();
        Ok(())
    })
}

pub(super) fn clear_ui(path: &Path, extension_id: Option<&str>) -> Result<(), String> {
    clear(path, true, extension_id)
}

fn clear(path: &Path, ui: bool, expected_id: Option<&str>) -> Result<(), String> {
    transaction(path, false, |read| {
        let mut journal = match read {
            JournalRead::Missing => return Ok(None),
            JournalRead::Valid(journal) => journal,
            JournalRead::Invalid => return Err(marker_error()),
        };
        let entry = if ui {
            journal.ui_mut()
        } else {
            journal.host_mut()
        };
        if expected_id.is_some_and(|expected| {
            entry
                .as_ref()
                .is_none_or(|current| current.extension_id != expected)
        }) {
            return Err(marker_error());
        }
        *entry = None;
        Ok(Some(journal))
    })
}

fn publish(path: &Path, journal: &LoadingJournal, fail_before_replace: bool) -> Result<(), String> {
    if journal.is_empty() {
        return remove(path);
    }
    ensure_safe_destination(path)?;
    let bytes = super::loading_journal_format::serialize(journal)?;
    if bytes.len() > MAX_MARKER_BYTES {
        return Err(marker_error());
    }
    #[cfg(test)]
    if fail_before_replace {
        return crate::services::private_store::atomic_write_fail_before_replace(path, &bytes)
            .map_err(|_| marker_error());
    }
    let _ = fail_before_replace;
    crate::services::private_store::atomic_write(path, &bytes).map_err(|_| {
        ::log::error!("[extensions] operation=loading-journal-write result=failed");
        marker_error()
    })
}

fn update(
    path: &Path,
    fail_before_replace: bool,
    change: impl FnOnce(&mut LoadingJournal) -> Result<(), String>,
) -> Result<(), String> {
    transaction(path, fail_before_replace, |read| {
        let mut journal = match read {
            JournalRead::Missing => LoadingJournal::empty(),
            JournalRead::Valid(journal) => journal,
            JournalRead::Invalid => return Err(marker_error()),
        };
        change(&mut journal)?;
        Ok(Some(journal))
    })
}

pub(super) fn transaction(
    path: &Path,
    fail_before_replace: bool,
    change: impl FnOnce(JournalRead) -> Result<Option<LoadingJournal>, String>,
) -> Result<(), String> {
    let _guard = MUTATION_LOCK.lock().map_err(|_| marker_error())?;
    let Some(journal) = change(read(path))? else {
        return Ok(());
    };
    publish(path, &journal, fail_before_replace)
}

pub(super) fn discard_host(path: &Path) -> Result<(), String> {
    transaction(path, false, |read| match read {
        JournalRead::Missing => Ok(None),
        JournalRead::Invalid => Ok(Some(LoadingJournal::empty())),
        JournalRead::Valid(mut journal) => {
            *journal.host_mut() = None;
            Ok(Some(journal))
        }
    })
}

pub(super) fn discard_invalid(path: &Path) -> Result<(), String> {
    transaction(path, false, |read| match read {
        JournalRead::Invalid => Ok(Some(LoadingJournal::empty())),
        JournalRead::Missing | JournalRead::Valid(_) => Err(marker_error()),
    })
}

fn ensure_safe_destination(path: &Path) -> Result<(), String> {
    match crate::services::private_store::open_regular_single_link(path) {
        Ok(None) => Ok(()),
        Ok(Some(file))
            if file
                .metadata()
                .is_ok_and(|metadata| metadata.len() <= MAX_MARKER_BYTES as u64) =>
        {
            Ok(())
        }
        Ok(Some(_)) | Err(_) => Err(marker_error()),
    }
}

fn remove(path: &Path) -> Result<(), String> {
    match crate::services::private_store::open_regular_single_link(path) {
        Ok(None) => Ok(()),
        Ok(Some(_)) => std::fs::remove_file(path).map_err(|_| marker_error()),
        Err(_) => Err(marker_error()),
    }
}
