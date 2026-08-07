use super::provider_error::SafeProviderDetails;
use serde::Serialize;
use std::path::Path;

const FILE_NAME: &str = "provider-errors.jsonl";
const MAX_LOG_BYTES: usize = 64 * 1024;
const MAX_IDENTIFIER_CHARS: usize = 128;

#[derive(Serialize)]
struct ProviderDiagnostic {
    timestamp: String,
    provider: String,
    model: String,
    status: u16,
    details: SafeProviderDetails,
    request_bytes: usize,
    tool_count: usize,
}

pub fn record_http_failure(
    provider: &str,
    model: &str,
    status: u16,
    details: SafeProviderDetails,
    request_bytes: usize,
    tool_count: usize,
) {
    let entry = ProviderDiagnostic {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider: safe_identifier(provider),
        model: safe_identifier(model),
        status,
        details,
        request_bytes,
        tool_count,
    };
    if write_at(&log_path(), &entry).is_err() {
        ::log::warn!("[llm] provider diagnostic log unavailable");
    }
}

fn safe_identifier(value: &str) -> String {
    let clipped: String = value.chars().take(MAX_IDENTIFIER_CHARS + 1).collect();
    if clipped.is_empty()
        || clipped.chars().count() > MAX_IDENTIFIER_CHARS
        || !clipped.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
        })
    {
        "unknown".to_string()
    } else {
        clipped
    }
}

fn log_path() -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join(FILE_NAME)
}

fn write_at(path: &Path, entry: &ProviderDiagnostic) -> Result<(), String> {
    let mut existing = bounded_existing(path)?;
    let mut line = serde_json::to_vec(entry).map_err(|_| "diagnostic unavailable".to_string())?;
    line.push(b'\n');
    while existing.len().saturating_add(line.len()) > MAX_LOG_BYTES {
        let Some(position) = existing.iter().position(|byte| *byte == b'\n') else {
            existing.clear();
            break;
        };
        existing.drain(..=position);
    }
    existing.extend_from_slice(&line);
    crate::services::private_store::atomic_write(path, &existing)
}

fn bounded_existing(path: &Path) -> Result<Vec<u8>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let metadata = std::fs::metadata(path).map_err(|_| "diagnostic unavailable".to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_LOG_BYTES as u64 {
        return Ok(Vec::new());
    }
    std::fs::read(path).map_err(|_| "diagnostic unavailable".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_is_bounded_and_contains_only_safe_fields() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join(FILE_NAME);
        let entry = ProviderDiagnostic {
            timestamp: "safe".to_string(),
            provider: safe_identifier("openai\nignored"),
            model: safe_identifier("gpt-5"),
            status: 400,
            details: SafeProviderDetails {
                error_type: Some("invalid_request".to_string()),
                error_code: Some("bad_schema".to_string()),
                error_param: Some("tools[0]".to_string()),
            },
            request_bytes: 100,
            tool_count: 2,
        };
        let mut line = serde_json::to_vec(&entry).unwrap();
        line.push(b'\n');
        let initial = line.repeat(MAX_LOG_BYTES / line.len());
        std::fs::write(&path, initial).unwrap();
        write_at(&path, &entry).unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.len() <= MAX_LOG_BYTES);
        assert!(!text.contains('\n') || text.ends_with('\n'));
        assert!(!text.contains("ignored"));
    }
}
