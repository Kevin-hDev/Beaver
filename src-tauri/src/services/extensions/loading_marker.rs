use std::path::{Path, PathBuf};

const FILE_NAME: &str = "extension-loading.json";
pub(super) const MAX_MARKER_BYTES: usize = 1_024;
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

pub(super) struct PreservedMarker {
    pub(super) state: MarkerRead,
    bytes: Option<Vec<u8>>,
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
    resolved_recovery_id: Option<&str>,
) -> Result<(), String> {
    complete_at(&path(), preserved, applied_ids, resolved_recovery_id)
}

pub(super) fn complete_at(
    marker_path: &Path,
    preserved: PreservedMarker,
    applied_ids: &std::collections::HashSet<String>,
    resolved_recovery_id: Option<&str>,
) -> Result<(), String> {
    let current = match read_at(marker_path) {
        MarkerRead::Valid(current) => current,
        MarkerRead::Missing if applied_ids.is_empty() => return Ok(()),
        MarkerRead::Invalid
            if applied_ids.is_empty() && matches!(preserved.state, MarkerRead::Invalid) =>
        {
            return Ok(())
        }
        MarkerRead::Missing | MarkerRead::Invalid => return Err(marker_error()),
    };
    if !applied_ids.contains(&current.extension_id) {
        return Ok(());
    }
    if resolved_recovery_id.is_some_and(|id| applied_ids.contains(id)) {
        return discard_at(marker_path);
    }
    match preserved.bytes {
        Some(bytes)
            if !matches!(
                preserved.state,
                MarkerRead::Valid(ref marker) if marker.extension_id == current.extension_id
            ) =>
        {
            write_bytes_at(marker_path, &bytes, false)
        }
        _ => discard_at(marker_path),
    }
}

pub(crate) fn read_at(path: &Path) -> MarkerRead {
    let bytes = match read_bytes_at(path) {
        Ok(crate::services::private_store::BoundedFile::Missing) => return MarkerRead::Missing,
        Ok(crate::services::private_store::BoundedFile::Content(bytes)) => bytes,
        Err(_) => return MarkerRead::Invalid,
    };
    super::loading_marker_format::parse(&bytes).map_or(MarkerRead::Invalid, MarkerRead::Valid)
}

pub(super) fn preserve_at(path: &Path) -> PreservedMarker {
    match read_bytes_at(path) {
        Ok(crate::services::private_store::BoundedFile::Missing) => PreservedMarker {
            state: MarkerRead::Missing,
            bytes: None,
        },
        Ok(crate::services::private_store::BoundedFile::Content(bytes)) => {
            let state = super::loading_marker_format::parse(&bytes)
                .map_or(MarkerRead::Invalid, MarkerRead::Valid);
            PreservedMarker {
                state,
                bytes: Some(bytes),
            }
        }
        Err(_) => PreservedMarker {
            state: MarkerRead::Invalid,
            bytes: None,
        },
    }
}

pub(crate) fn start_at(path: &Path, extension_id: &str, attempts: u8) -> Result<(), String> {
    let marker = LoadingMarker::new(extension_id, attempts)?;
    write_at(path, &marker, false)
}

pub(super) fn advance_at(path: &Path, extension_id: &str, stage: &str) -> Result<(), String> {
    let MarkerRead::Valid(mut marker) = read_at(path) else {
        return Err(marker_error());
    };
    if marker.extension_id != extension_id || !super::types::HOST_LOAD_STAGES.contains(&stage) {
        return Err(marker_error());
    }
    marker.stage = stage.to_string();
    write_at(path, &marker, false)
}

#[cfg(test)]
pub(super) fn advance_fail_before_replace_at(
    path: &Path,
    extension_id: &str,
    stage: &str,
) -> Result<(), String> {
    let MarkerRead::Valid(mut marker) = read_at(path) else {
        return Err(marker_error());
    };
    if marker.extension_id != extension_id || !super::types::HOST_LOAD_STAGES.contains(&stage) {
        return Err(marker_error());
    }
    marker.stage = stage.to_string();
    write_at(path, &marker, true)
}

pub(super) fn discard_at(path: &Path) -> Result<(), String> {
    match crate::services::private_store::open_regular_single_link(path) {
        Ok(None) => Ok(()),
        Ok(Some(_)) => std::fs::remove_file(path).map_err(|_| marker_error()),
        Err(_) => Err(marker_error()),
    }
}

#[cfg(test)]
pub(super) fn discard_invalid_at(path: &Path) -> Result<(), String> {
    if !matches!(read_at(path), MarkerRead::Invalid) {
        return Err(marker_error());
    }
    discard_at(path)
}

fn write_at(path: &Path, marker: &LoadingMarker, fail_before_replace: bool) -> Result<(), String> {
    ensure_safe_destination(path)?;
    let bytes = super::loading_marker_format::serialize(marker)?;
    if bytes.len() > MAX_MARKER_BYTES {
        return Err(marker_error());
    }
    write_bytes_at(path, &bytes, fail_before_replace)
}

fn write_bytes_at(path: &Path, bytes: &[u8], fail_before_replace: bool) -> Result<(), String> {
    ensure_safe_destination(path)?;
    if bytes.len() > MAX_MARKER_BYTES {
        return Err(marker_error());
    }
    #[cfg(test)]
    if fail_before_replace {
        return crate::services::private_store::atomic_write_fail_before_replace(path, bytes)
            .map_err(|_| marker_error());
    }
    let _ = fail_before_replace;
    crate::services::private_store::atomic_write(path, bytes).map_err(|_| marker_error())
}

fn read_bytes_at(path: &Path) -> Result<crate::services::private_store::BoundedFile, String> {
    crate::services::private_store::read_bounded_regular(path, MAX_MARKER_BYTES as u64)
}

fn ensure_safe_destination(path: &Path) -> Result<(), String> {
    match crate::services::private_store::open_regular_single_link(path) {
        Ok(None) | Ok(Some(_)) => Ok(()),
        Err(_) => Err(marker_error()),
    }
}

fn path() -> PathBuf {
    crate::services::paths::data_dir().join(FILE_NAME)
}

fn marker_error() -> String {
    super::error_codes::RECOVERY_MARKER_INVALID.to_string()
}
