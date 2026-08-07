use reqwest::{Response, StatusCode};
use zeroize::Zeroizing;

use crate::services::llm::provider_error::ProviderErrorCode;
use crate::services::secure_http::{read_bounded, PROVIDER_ERROR_LIMIT};

pub async fn require_success(
    response: Response,
    model: &str,
    request_bytes: usize,
    tool_count: usize,
) -> Result<Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = read_error_body(response).await;
    crate::services::llm::provider_diagnostics::record_http_failure(
        "codex-oauth",
        model,
        status.as_u16(),
        crate::services::llm::provider_error::safe_details(&body),
        request_bytes,
        tool_count,
    );
    let safe_code = safe_status_code(status);
    ::log::warn!("[codex stream] HTTP {status} code={safe_code}");
    Err(status_error(status, &body))
}

pub fn stream_failure(event: &serde_json::Value) -> String {
    let code = event
        .pointer("/response/error/code")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();

    if matches!(code, "rate_limit" | "rate_limit_exceeded") {
        return "rate_limit".to_string();
    }
    if is_temporary_provider_code(code) {
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
        429 => "rate_limit".to_string(),
        413 => "provider_payload_too_large".to_string(),
        500..=599 => temporarily_unavailable(),
        _ if body_has_temporary_code(body) => temporarily_unavailable(),
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

fn is_temporary_provider_code(value: &str) -> bool {
    matches!(
        value,
        "server_error"
            | "service_unavailable"
            | "temporarily_unavailable"
            | "overloaded"
            | "circuit_open"
    )
}

fn body_has_temporary_code(body: &str) -> bool {
    crate::services::llm::provider_error::safe_details(body)
        .error_code
        .as_deref()
        .is_some_and(is_temporary_provider_code)
}

#[cfg(test)]
#[path = "http_error_tests.rs"]
mod tests;
