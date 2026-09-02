use serde::Serialize;
use std::path::Path;

#[cfg(not(test))]
const FILE_NAME: &str = "extension-install.jsonl";

#[derive(Serialize)]
struct Entry<'a> {
    timestamp: String,
    operation: &'a str,
    code: &'a str,
    reason: &'a str,
}

#[cfg(not(test))]
pub fn write(operation: &str, code: &str, reason: &str) {
    let reason = safe_reason(reason);
    if write_at(&log_path(), operation, code, reason).is_err() {
        ::log::error!("[extensions] operation failed; diagnostic log unavailable");
    } else {
        ::log::error!("[extensions] operation failed: {operation}/{code}/{reason}");
    }
}

#[cfg(test)]
pub fn write(operation: &str, code: &str, reason: &str) {
    let _ = (operation, code, reason);
}

#[cfg(not(test))]
fn log_path() -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join(FILE_NAME)
}

fn write_at(path: &Path, operation: &str, code: &str, reason: &str) -> Result<(), String> {
    let reason = safe_reason(reason);
    let entry = Entry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation,
        code,
        reason,
    };
    super::bounded_jsonl::write(path, &entry)
}

fn safe_reason(reason: &str) -> &str {
    if super::operation_error::is_safe_reason(reason) {
        reason
    } else {
        "operation_failed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_is_bounded_and_contains_only_safe_fields() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("operations.jsonl");
        let line =
            "{\"timestamp\":\"safe\",\"operation\":\"install_git\",\"code\":\"extensions_install_failed\"}\n";
        let initial = line.repeat(super::super::bounded_jsonl::MAX_LOG_BYTES / line.len());
        std::fs::write(&path, initial).unwrap();
        write_at(
            &path,
            "install_git",
            "extensions_git_download_failed",
            "secret-sentinel https://private /Users/private\nstack backtrace",
        )
        .unwrap();
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() <= super::super::bounded_jsonl::MAX_LOG_BYTES);
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("https://"));
        assert!(!text.contains("/Users/"));
        assert!(!text.contains("secret-sentinel"));
        assert!(!text.contains("stack"));
        assert!(text.contains("operation_failed"));
    }
}
