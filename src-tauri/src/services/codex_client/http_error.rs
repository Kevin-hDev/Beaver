use reqwest::{Response, StatusCode};
use zeroize::Zeroizing;

use crate::services::llm::provider_error::ProviderErrorCode;
use crate::services::secure_http::{read_bounded, PROVIDER_ERROR_LIMIT};

pub async fn require_success(response: Response) -> Result<Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = read_error_body(response).await;
    let safe_code = safe_status_code(status);
    eprintln!("[codex stream] HTTP {status} code={safe_code}");
    Err(status_error(status, &body))
}

pub fn stream_failure(event: &serde_json::Value) -> String {
    let code = event
        .pointer("/response/error/code")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let message = event
        .pointer("/response/error/message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let lower = Zeroizing::new(format!("{code} {message}").to_ascii_lowercase());

    if lower.contains("rate_limit") || lower.contains("rate limit") {
        return "Codex rate limit".to_string();
    }
    if is_temporary_provider_failure(&lower) {
        return temporarily_unavailable();
    }
    "provider_request_rejected".to_string()
}

async fn read_error_body(response: Response) -> Zeroizing<String> {
    match read_bounded(response, PROVIDER_ERROR_LIMIT).await {
        Ok(bytes) => Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => Zeroizing::new(String::new()),
    }
}

fn status_error(status: StatusCode, body: &str) -> String {
    match status.as_u16() {
        401 => "oauth_reauthentication_required".to_string(),
        403 => "provider_access_unavailable".to_string(),
        429 => "Codex HTTP 429 rate limit".to_string(),
        500..=599 => temporarily_unavailable(),
        _ if body_is_temporary_failure(body) => temporarily_unavailable(),
        _ => "provider_request_rejected".to_string(),
    }
}

fn temporarily_unavailable() -> String {
    ProviderErrorCode::ProviderTemporarilyUnavailable
        .as_str()
        .to_string()
}

fn safe_status_code(status: StatusCode) -> &'static str {
    match status.as_u16() {
        401 => "authentication_required",
        403 => "provider_access_unavailable",
        429 => "rate_limit",
        500..=599 => "provider_temporarily_unavailable",
        _ => "provider_request_rejected",
    }
}

fn is_temporary_provider_failure(value: &str) -> bool {
    [
        "high demand",
        "overloaded",
        "service unavailable",
        "temporarily unavailable",
        "circuit_open",
        "circuit open",
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

fn body_is_temporary_failure(body: &str) -> bool {
    let lower = Zeroizing::new(body.to_ascii_lowercase());
    is_temporary_provider_failure(&lower)
}

#[cfg(test)]
#[path = "http_error_tests.rs"]
mod tests;
