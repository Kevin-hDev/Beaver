use reqwest::Response;

use crate::services::llm::provider_error::ProviderErrorCode;

pub async fn require_success(
    response: Response,
    model: &str,
    request_bytes: usize,
    tool_count: usize,
    request_id: Option<&str>,
    serialized_request: &str,
) -> Result<Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    Err(crate::services::llm::stream_http::reject_response(
        response,
        crate::services::llm::stream_http::RejectedResponseContext {
            provider_id: "codex-oauth",
            model,
            error_policy: crate::services::llm::route_profile::ErrorPolicy::Codex,
            oauth: true,
            request_bytes,
            tool_count,
            diagnostic:
                crate::services::llm::provider_diagnostics::ProviderDiagnosticContext::from_serialized(
                    request_id,
                    serialized_request,
                ),
        },
    )
    .await
    .to_string())
}

pub fn stream_failure(event: &serde_json::Value) -> String {
    if crate::services::llm::provider_error::is_service_tier_response_error(event) {
        return service_tier_unavailable();
    }
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

fn service_tier_unavailable() -> String {
    ProviderErrorCode::ServiceTierUnavailable
        .as_str()
        .to_string()
}

fn temporarily_unavailable() -> String {
    ProviderErrorCode::ProviderTemporarilyUnavailable
        .as_str()
        .to_string()
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

#[cfg(test)]
#[path = "http_error_tests.rs"]
mod tests;
