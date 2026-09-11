use crate::services::llm::stream_http::{RequestConfig, RequestError};
use crate::services::secure_http::{read_bounded, PROVIDER_ERROR_LIMIT};

pub(super) async fn classify_response(
    response: reqwest::Response,
    route: &crate::services::llm::route::LlmRoute,
    config: &RequestConfig<'_>,
    request_bytes: usize,
    request_id: Option<&str>,
    payload: &serde_json::Value,
) -> Result<reqwest::Response, RequestError> {
    let status = response.status();
    let has_retry_after = response.headers().contains_key("retry-after");
    let diagnostic_context =
        crate::services::llm::provider_diagnostics::ProviderDiagnosticContext::from_payload(
            request_id, payload,
        )
        .with_retry_after(response.headers());
    let body = read_bounded(response, PROVIDER_ERROR_LIMIT)
        .await
        .map(|bytes| zeroize::Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()))
        .unwrap_or_default();
    let code = crate::services::llm::provider_error::safe_log_code(
        route.error_policy,
        status.as_u16(),
        &body,
    );
    crate::services::llm::provider_diagnostics::record_http_failure(
        config.provider_id,
        config.model,
        status.as_u16(),
        crate::services::llm::provider_error::safe_details(&body),
        request_bytes,
        config.tools.len(),
        diagnostic_context,
    );
    ::log::warn!("[anthropic messages] HTTP {status} code={code}");
    Err(crate::services::llm::stream_http::classify_error(
        status.as_u16(),
        &body,
        route.display_name,
        route.error_policy,
        false,
        has_retry_after,
    ))
}

pub(super) fn map_route_error(error: crate::services::llm::route::RouteError) -> RequestError {
    match error {
        crate::services::llm::route::RouteError::Unauthorized => {
            RequestError::Fatal("auth_failed".into())
        }
        crate::services::llm::route::RouteError::Forbidden => {
            RequestError::Fatal("provider_access_unavailable".into())
        }
        crate::services::llm::route::RouteError::Network => {
            RequestError::Fatal("provider_connection_failed".into())
        }
        #[cfg(debug_assertions)]
        crate::services::llm::route::RouteError::FixtureBudget(message) => {
            RequestError::Fatal(message)
        }
    }
}
