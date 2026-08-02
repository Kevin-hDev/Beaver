use serde::Serialize;
use std::path::Path;

const PATH_STATUS_FILE: &str = "shell-environment.json";
const XCRUN_FAILURE_FILE: &str = "shell-xcrun-error.json";

#[derive(Clone, Copy)]
enum Status {
    PathCaptured,
    PathFallback,
    XcrunUnavailable,
}

#[derive(Serialize)]
struct Entry<'a> {
    timestamp: String,
    status: &'a str,
}

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

fn record(file: &str, status: Status) {
    if write_at(&log_path(file), status).is_err() {
        eprintln!("[shell] persistent diagnostic unavailable");
    }
}

fn write_at(path: &Path, status: Status) -> Result<(), String> {
    let entry = Entry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        status: match status {
            Status::PathCaptured => "path_captured",
            Status::PathFallback => "path_fallback",
            Status::XcrunUnavailable => "xcrun_unavailable",
        },
    };
    let bytes = serde_json::to_vec(&entry).map_err(|_| "diagnostic unavailable".to_string())?;
    crate::services::private_store::atomic_write(path, &bytes)
}

fn log_path(file: &str) -> std::path::PathBuf {
    crate::services::paths::data_dir().join("logs").join(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_is_bounded_and_contains_no_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("status.json");

        write_at(&path, Status::PathFallback).expect("diagnostic");

        let bytes = std::fs::read(path).expect("read");
        assert!(bytes.len() < 256);
        let text = String::from_utf8(bytes).expect("utf8");
        assert!(text.contains("path_fallback"));
        assert!(!text.contains("/Users/"));
    }
}
