use serde::Serialize;
use std::path::Path;

const FILE_NAME: &str = "extension-install.jsonl";
const MAX_LOG_BYTES: usize = 64 * 1024;
const MAX_EXISTING_BYTES: u64 = MAX_LOG_BYTES as u64;

#[derive(Serialize)]
struct Entry<'a> {
    timestamp: String,
    operation: &'a str,
    code: &'a str,
    reason: &'a str,
}

pub fn write(operation: &str, code: &str, reason: &str) {
    if write_at(&log_path(), operation, code, reason).is_err() {
        eprintln!("[extensions] operation failed; diagnostic log unavailable");
    } else {
        eprintln!("[extensions] operation failed: {operation}/{code}/{reason}");
    }
}

fn log_path() -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join(FILE_NAME)
}

fn write_at(path: &Path, operation: &str, code: &str, reason: &str) -> Result<(), String> {
    let entry = Entry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation,
        code,
        reason,
    };
    let mut next = bounded_existing(path)?;
    let mut line =
        serde_json::to_vec(&entry).map_err(|_| "journal d'extensions indisponible".to_string())?;
    line.push(b'\n');
    if line.len() > MAX_LOG_BYTES {
        return Err("journal d'extensions indisponible".to_string());
    }
    while next.len() + line.len() > MAX_LOG_BYTES {
        let Some(position) = next.iter().position(|byte| *byte == b'\n') else {
            next.clear();
            break;
        };
        next.drain(..=position);
    }
    next.extend_from_slice(&line);
    crate::services::private_store::atomic_write(path, &next)
}

fn bounded_existing(path: &Path) -> Result<Vec<u8>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let metadata =
        std::fs::metadata(path).map_err(|_| "journal d'extensions indisponible".to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_EXISTING_BYTES {
        return Ok(Vec::new());
    }
    std::fs::read(path).map_err(|_| "journal d'extensions indisponible".to_string())
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
        let initial = line.repeat(MAX_LOG_BYTES / line.len());
        std::fs::write(&path, initial).unwrap();
        write_at(
            &path,
            "install_git",
            "extensions_git_download_failed",
            "operation_failed",
        )
        .unwrap();
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() <= MAX_LOG_BYTES);
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("https://"));
        assert!(!text.contains("/Users/"));
    }
}
