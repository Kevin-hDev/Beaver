use crate::services::agent_local::types_ollama::{StreamOutcome, StreamResult};
use crate::services::llm::stream_http::{RequestConfig, RequestError};
use crate::services::secure_http::{read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT};
use tokio_util::sync::CancellationToken;

#[allow(
    clippy::too_many_arguments,
    reason = "stream runtime dependencies remain explicit"
)]
pub(in crate::services::llm) async fn stream_chat(
    on_event: &crate::services::agent_local::stream_events::AgentEventEmitter,
    config: &RequestConfig<'_>,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
    reasoning_capture: Option<crate::services::llm::reasoning_wire::ReasoningCapture>,
    request_id: &str,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    let response = post(config, measurement.as_deref_mut(), Some(request_id))
        .await
        .map_err(|error| error.to_string())?;
    super::stream::consume_stream(
        on_event,
        response,
        cancel,
        buffer_content,
        realtime_budget,
        config.tools,
        crate::services::provider_usage::UsageContext {
            canonical_provider_id: "anthropic",
            model: config.model,
            api_format: crate::services::provider_usage::UsageApiFormat::AnthropicMessages,
        },
        reasoning_capture,
        measurement,
    )
    .await
}

pub(in crate::services::llm) async fn collect_silent(
    config: &RequestConfig<'_>,
    cancel: CancellationToken,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    let response = post(config, measurement.as_deref_mut(), None)
        .await
        .map_err(|error| error.to_string())?;
    super::stream::consume_silent(
        response,
        cancel,
        crate::services::provider_usage::UsageContext {
            canonical_provider_id: "anthropic",
            model: config.model,
            api_format: crate::services::provider_usage::UsageApiFormat::AnthropicMessages,
        },
        measurement,
    )
    .await
}

async fn post(
    config: &RequestConfig<'_>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
    request_id: Option<&str>,
) -> Result<reqwest::Response, RequestError> {
    let route = crate::services::llm::route::resolve(config.provider_id)
        .ok_or(RequestError::InvalidConfiguration)?;
    let estimated = crate::services::compress::token_estimate::estimate_request_tokens_for_provider(
        config.provider_id,
        config.messages,
        config.tools,
    );
    let max_tokens = crate::services::llm::stream_max_tokens::resolve(
        route.canonical_provider_id,
        config.model,
        config.max_tokens,
        route.auto_max_tokens,
        route.fallback_max_tokens,
        estimated,
    )
    .await
    .map_err(|error| match error {
        crate::services::llm::stream_max_tokens::ResolveError::ContextExhausted => {
            RequestError::PayloadTooLarge
        }
        crate::services::llm::stream_max_tokens::ResolveError::InvalidLimit => {
            RequestError::InvalidConfiguration
        }
    })?
    .ok_or(RequestError::InvalidConfiguration)?;
    let prepared =
        super::build_payload(config, max_tokens).map_err(|_| RequestError::InvalidConfiguration)?;
    let request_bytes = serde_json::to_vec(&prepared.payload)
        .map(zeroize::Zeroizing::new)
        .map_err(|_| RequestError::InvalidConfiguration)?
        .len();
    crate::services::llm::reasoning_wire::replay::record_evidence(
        config.session_id,
        request_id,
        &prepared.replayed,
    )
    .await;
    #[cfg(test)]
    if let Some(response) =
        crate::services::llm::stream_test_transport::dispatch(config, &prepared.payload).await
    {
        return response;
    }
    let client = AuthenticatedClient::new_streaming(
        crate::services::llm::timeouts::connect_timeout(),
        crate::services::llm::timeouts::idle_timeout_for(config.provider_id),
    )
    .map_err(|_| RequestError::InvalidConfiguration)?;
    let url = format!("{}/messages", route.base_url);
    let (header, static_headers) =
        super::client::auth_headers().map_err(|_| RequestError::InvalidConfiguration)?;
    let usage_generation =
        crate::services::provider_usage::credential_generation(config.provider_id);
    let response = route
        .send_authenticated(&client, config.purpose, |token, inherited| {
            let request = client.post(&url).headers(inherited).json(&prepared.payload);
            let request = crate::services::llm::request_auth::apply(request, header, token);
            static_headers
                .iter()
                .fold(request, |request, (name, value)| {
                    request.header(*name, *value)
                })
        })
        .await
        .map_err(map_route_error)?;
    if let Some(measurement) = measurement.as_mut() {
        measurement.mark_headers();
    }
    crate::services::provider_usage::capture_headers(
        config.provider_id,
        usage_generation,
        response.headers(),
    )
    .await;
    if response.status().is_success() {
        return Ok(response);
    }
    classify_response(response, &route, config, request_bytes).await
}

async fn classify_response(
    response: reqwest::Response,
    route: &crate::services::llm::route::LlmRoute,
    config: &RequestConfig<'_>,
    request_bytes: usize,
) -> Result<reqwest::Response, RequestError> {
    let status = response.status();
    let has_retry_after = response.headers().contains_key("retry-after");
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

fn map_route_error(error: crate::services::llm::route::RouteError) -> RequestError {
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
    }
}
