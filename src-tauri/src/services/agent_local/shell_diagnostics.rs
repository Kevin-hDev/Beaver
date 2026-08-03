use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

const PATH_STATUS_FILE: &str = "shell-environment.json";
const XCRUN_FAILURE_FILE: &str = "shell-xcrun-error.json";
const ROOT_LIMIT_FILE: &str = "shell-root-limit.json";
const MAX_DIAGNOSTIC_BYTES: u64 = 256;

#[derive(Clone, Copy)]
enum Status {
    PathCaptured,
    PathFallback,
    XcrunUnavailable,
    RootPath,
    RootRead,
    RootWrite,
    RootPathRead,
    RootPathWrite,
    RootReadWrite,
    RootAll,
}

#[derive(Deserialize, Serialize)]
struct Entry {
    timestamp: String,
    status: String,
}

pub struct RootLimitMarker(SystemTime);

pub fn record_path_capture(captured: bool) {
    let status = if captured {
        Status::PathCaptured
    } else {
        Status::PathFallback
    };
    record(PATH_STATUS_FILE, status);
}

pub fn record_xcrun_failure() {
    record(XCRUN_FAILURE_FILE, Status::XcrunUnavailable);
}

pub fn clear_xcrun_failure() {
    let path = log_path(XCRUN_FAILURE_FILE);
    if path.is_file() && std::fs::remove_file(path).is_err() {
        eprintln!("[shell] diagnostic cleanup unavailable");
    }
}

pub fn record_root_limits(path: bool, read: bool, write: bool) {
    let diagnostic = log_path(ROOT_LIMIT_FILE);
    match root_status(path, read, write) {
        Some(status) => {
            let _ = write_at(&diagnostic, status);
        }
        None => {
            let _ = clear_at(&diagnostic);
        }
    }
}

pub fn root_limit_marker() -> RootLimitMarker {
    RootLimitMarker(SystemTime::now())
}

pub fn root_limit_warning_since(marker: &RootLimitMarker) -> Option<String> {
    warning_since_at(&log_path(ROOT_LIMIT_FILE), marker)
}

fn record(file: &str, status: Status) {
    if write_at(&log_path(file), status).is_err() {
        eprintln!("[shell] persistent diagnostic unavailable");
    }
}

fn write_at(path: &Path, status: Status) -> Result<(), String> {
    let entry = Entry {
        timestamp: chrono::Utc::now()
            .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        status: status_name(status).to_string(),
    };
    let bytes = serde_json::to_vec(&entry).map_err(|_| "diagnostic unavailable".to_string())?;
    if bytes.len() >= MAX_DIAGNOSTIC_BYTES as usize {
        return Err("diagnostic unavailable".to_string());
    }
    crate::services::private_store::atomic_write(path, &bytes)
}

fn warning_since_at(path: &Path, marker: &RootLimitMarker) -> Option<String> {
    let metadata = safe_metadata(path)?;
    if metadata.modified().ok()? < marker.0 {
        return None;
    }
    let current = read_with_metadata(path, &metadata)?;
    let entry: Entry = serde_json::from_slice(&current).ok()?;
    let (path, read, write) = root_flags(&entry.status)?;
    let mut discarded = Vec::with_capacity(3);
    if path {
        discarded.push("des entrées excédentaires du PATH et leurs caches inscriptibles");
    }
    if read {
        discarded.push("des racines d’outils en lecture seule");
    }
    if write {
        discarded.push("des racines d’outils en écriture");
    }
    Some(format!(
        "Le bac à sable a écarté {} parce qu’un plafond de configuration a été atteint ; certains outils ou caches peuvent être indisponibles pour cette commande.",
        discarded.join(", ")
    ))
}

fn safe_metadata(path: &Path) -> Option<std::fs::Metadata> {
    let metadata = path.symlink_metadata().ok()?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() >= MAX_DIAGNOSTIC_BYTES
    {
        return None;
    }
    Some(metadata)
}

fn read_with_metadata(path: &Path, metadata: &std::fs::Metadata) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).ok()?;
    (bytes.len() < MAX_DIAGNOSTIC_BYTES as usize && bytes.len() as u64 == metadata.len())
        .then_some(bytes)
}

fn clear_at(path: &Path) -> Result<(), String> {
    let Ok(metadata) = path.symlink_metadata() else {
        return Ok(());
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("diagnostic unavailable".to_string());
    }
    std::fs::remove_file(path).map_err(|_| "diagnostic unavailable".to_string())
}

fn root_status(path: bool, read: bool, write: bool) -> Option<Status> {
    match (path, read, write) {
        (false, false, false) => None,
        (true, false, false) => Some(Status::RootPath),
        (false, true, false) => Some(Status::RootRead),
        (false, false, true) => Some(Status::RootWrite),
        (true, true, false) => Some(Status::RootPathRead),
        (true, false, true) => Some(Status::RootPathWrite),
        (false, true, true) => Some(Status::RootReadWrite),
        (true, true, true) => Some(Status::RootAll),
    }
}

fn status_name(status: Status) -> &'static str {
    match status {
        Status::PathCaptured => "path_captured",
        Status::PathFallback => "path_fallback",
        Status::XcrunUnavailable => "xcrun_unavailable",
        Status::RootPath => "root_path_limit",
        Status::RootRead => "root_read_limit",
        Status::RootWrite => "root_write_limit",
        Status::RootPathRead => "root_path_read_limit",
        Status::RootPathWrite => "root_path_write_limit",
        Status::RootReadWrite => "root_read_write_limit",
        Status::RootAll => "root_all_limits",
    }
}

fn root_flags(status: &str) -> Option<(bool, bool, bool)> {
    match status {
        "root_path_limit" => Some((true, false, false)),
        "root_read_limit" => Some((false, true, false)),
        "root_write_limit" => Some((false, false, true)),
        "root_path_read_limit" => Some((true, true, false)),
        "root_path_write_limit" => Some((true, false, true)),
        "root_read_write_limit" => Some((false, true, true)),
        "root_all_limits" => Some((true, true, true)),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn record_root_limits_for_test(
    path_limit: bool,
    read: bool,
    write: bool,
    file: &Path,
) {
    if let Some(status) = root_status(path_limit, read, write) {
        write_at(file, status).expect("root limit diagnostic");
    }
}

fn log_path(file: &str) -> std::path::PathBuf {
    crate::services::paths::data_dir().join("logs").join(file)
}

#[cfg(test)]
#[path = "shell_diagnostics_tests.rs"]
mod tests;
