//! A successful HTTP handshake can still carry a structured stream failure.
//! Keep its safe fields in the existing bounded journal, never its free-form body.

use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct StreamDiagnostic {
    timestamp: String,
    transport: &'static str,
    provider: String,
    model: String,
    request_id: Option<String>,
    details: super::SafeProviderDetails,
}

#[derive(Serialize)]
struct ErrorDocument<'a> {
    error: &'a Value,
}

pub(crate) fn record_stream_failure(provider: &str, model: &str, request_id: &str, event: &Value) {
    if !matches!(event["type"].as_str(), Some("response.failed" | "error")) {
        return;
    }
    let error = event
        .pointer("/response/error")
        .or_else(|| event.get("error"))
        .unwrap_or(event);
    let body = match serde_json::to_string(&ErrorDocument { error }) {
        Ok(body) => zeroize::Zeroizing::new(body),
        Err(_) => {
            log::warn!("[llm] provider diagnostic serialization unavailable");
            return;
        }
    };
    let entry = StreamDiagnostic {
        timestamp: chrono::Utc::now().to_rfc3339(),
        transport: "stream",
        provider: super::safe_identifier(provider),
        model: super::safe_model_identifier(model),
        request_id: super::safe_request_id(request_id),
        details: crate::services::llm::provider_error::safe_details(&body),
    };
    if super::write_at(&super::log_path(), &entry).is_err() {
        log::warn!("[llm] provider diagnostic log unavailable");
    }
}
