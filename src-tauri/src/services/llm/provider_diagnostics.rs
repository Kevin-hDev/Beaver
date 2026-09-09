use super::provider_error::SafeProviderDetails;
use serde::Serialize;
use std::path::Path;

#[path = "provider_diagnostics_stream.rs"]
mod stream;
pub(crate) use stream::record_stream_failure;

const FILE_NAME: &str = "provider-errors.jsonl";
const MAX_LOG_BYTES: usize = 64 * 1024;
const MAX_IDENTIFIER_CHARS: usize = 128;
static WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Serialize)]
struct ProviderDiagnostic {
    timestamp: String,
    provider: String,
    model: String,
    status: u16,
    details: SafeProviderDetails,
    request_bytes: usize,
    tool_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_limit: Option<SerializedOutputLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after_seconds: Option<u64>,
}

#[derive(Clone, Serialize)]
struct SerializedOutputLimit {
    field: &'static str,
    value: u64,
}

pub(crate) struct ProviderDiagnosticContext {
    request_id: Option<String>,
    output_limit: Option<SerializedOutputLimit>,
    retry_after_seconds: Option<u64>,
}

impl ProviderDiagnosticContext {
    pub(crate) fn from_payload(request_id: Option<&str>, payload: &serde_json::Value) -> Self {
        const OUTPUT_FIELDS: [&str; 3] =
            ["max_output_tokens", "max_completion_tokens", "max_tokens"];
        let output_limit = OUTPUT_FIELDS.iter().find_map(|field| {
            payload
                .get(*field)
                .and_then(serde_json::Value::as_u64)
                .filter(|value| *value > 0)
                .map(|value| SerializedOutputLimit { field, value })
        });
        Self {
            request_id: request_id.and_then(safe_request_id),
            output_limit,
            retry_after_seconds: None,
        }
    }

    pub(crate) fn with_retry_after(mut self, headers: &reqwest::header::HeaderMap) -> Self {
        self.retry_after_seconds = super::provider_error::retry_after_seconds(headers);
        self
    }

    pub(crate) fn from_serialized(request_id: Option<&str>, payload: &str) -> Self {
        serde_json::from_str(payload).map_or_else(
            |_| Self::from_payload(request_id, &serde_json::Value::Null),
            |value| Self::from_payload(request_id, &value),
        )
    }
}

pub fn record_http_failure(
    provider: &str,
    model: &str,
    status: u16,
    details: SafeProviderDetails,
    request_bytes: usize,
    tool_count: usize,
    context: ProviderDiagnosticContext,
) {
    let entry = ProviderDiagnostic {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider: safe_identifier(provider),
        model: safe_model_identifier(model),
        status,
        details,
        request_bytes,
        tool_count,
        request_id: context.request_id,
        output_limit: context.output_limit,
        retry_after_seconds: context.retry_after_seconds,
    };
    if write_at(&log_path(), &entry).is_err() {
        ::log::warn!("[llm] provider diagnostic log unavailable");
    }
}

fn safe_request_id(value: &str) -> Option<String> {
    let clipped: String = value.chars().take(MAX_IDENTIFIER_CHARS + 1).collect();
    (!clipped.is_empty()
        && clipped.chars().count() <= MAX_IDENTIFIER_CHARS
        && clipped
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-')))
    .then_some(clipped)
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

fn safe_model_identifier(value: &str) -> String {
    if crate::services::model_identifier::is_valid_model_id(value) {
        value.to_string()
    } else {
        "unknown".to_string()
    }
}

fn log_path() -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join(FILE_NAME)
}

fn write_at(path: &Path, entry: &impl Serialize) -> Result<(), String> {
    // Atomic replacement alone loses entries when HTTP/stream failures overlap.
    let _guard = WRITE_LOCK
        .lock()
        .map_err(|_| "diagnostic unavailable".to_string())?;
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
#[path = "provider_diagnostics_tests.rs"]
mod tests;
