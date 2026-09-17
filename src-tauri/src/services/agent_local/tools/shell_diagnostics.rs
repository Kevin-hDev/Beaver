use serde::{Deserialize, Serialize};
use std::path::Path;

const PATH_STATUS_FILE: &str = "shell-environment.json";
#[cfg(target_os = "macos")]
const XCRUN_FAILURE_FILE: &str = "shell-xcrun-error.json";
const MAX_DIAGNOSTIC_BYTES: u64 = 256;

#[derive(Clone, Copy)]
enum Status {
    PathCaptured,
    PathFallback,
    #[cfg(target_os = "macos")]
    XcrunUnavailable,
}

#[derive(Deserialize, Serialize)]
struct Entry {
    timestamp: String,
    status: String,
}

pub fn record_path_capture(captured: bool) {
    let status = if captured {
        Status::PathCaptured
    } else {
        Status::PathFallback
    };
    record(PATH_STATUS_FILE, status);
}

#[cfg(target_os = "macos")]
pub fn record_xcrun_failure() {
    record(XCRUN_FAILURE_FILE, Status::XcrunUnavailable);
}

#[cfg(target_os = "macos")]
pub fn clear_xcrun_failure() {
    let path = log_path(XCRUN_FAILURE_FILE);
    if path.is_file() && std::fs::remove_file(path).is_err() {
        ::log::warn!("[shell] diagnostic cleanup unavailable");
    }
}

fn record(file: &str, status: Status) {
    if write_at(&log_path(file), status).is_err() {
        ::log::warn!("[shell] persistent diagnostic unavailable");
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

fn status_name(status: Status) -> &'static str {
    match status {
        Status::PathCaptured => "path_captured",
        Status::PathFallback => "path_fallback",
        #[cfg(target_os = "macos")]
        Status::XcrunUnavailable => "xcrun_unavailable",
    }
}

fn log_path(file: &str) -> std::path::PathBuf {
    crate::services::paths::data_dir().join("logs").join(file)
}

#[cfg(test)]
#[path = "shell_diagnostics_tests.rs"]
mod tests;
