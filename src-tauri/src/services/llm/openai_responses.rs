use crate::services::agent_local::types_ollama::StreamOutcome;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::secure_http::AuthenticatedClient;
use tokio_util::sync::CancellationToken;

use super::stream_http::{RequestConfig, RequestError};

#[path = "openai_responses_request.rs"]
mod request_builder;
#[cfg(test)]
pub(super) use request_builder::build_request;
#[cfg(test)]
use request_builder::try_build_request;
use request_builder::try_build_request_with_evidence;

pub(super) struct ResponseStreamOptions<'a> {
    pub buffer_content: bool,
    pub realtime_budget: Option<RealtimeBudget>,
    pub reasoning_capture: Option<super::reasoning_wire::ReasoningCapture>,
    pub request_id: &'a str,
    pub preparation:
        Option<&'a crate::services::agent_local::context_usage_runtime::PreparedContextAttempt<'a>>,
}

pub(super) async fn stream_chat(
    on_event: &crate::services::agent_local::stream_events::AgentEventEmitter,
    config: &RequestConfig<'_>,
    cancel: CancellationToken,
    options: ResponseStreamOptions<'_>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    let response = post(
        config,
        measurement.as_deref_mut(),
        Some(options.request_id),
        options.preparation,
    )
    .await
    .map_err(request_error)?;
    crate::services::codex_client::stream::consume_external_responses_sse(
        on_event,
        response,
        cancel,
        options.buffer_content,
        options.realtime_budget,
        config.provider_id,
        config.model,
        config.tools,
        options.reasoning_capture,
        measurement,
    )
    .await
}

pub(super) async fn collect_silent(
    config: &RequestConfig<'_>,
    cancel: CancellationToken,
    max_text_bytes: usize,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    let response = post(config, measurement.as_deref_mut(), None, None)
        .await
        .map_err(request_error)?;
    crate::services::codex_client::stream_silent::consume_external_responses_sse_silent(
        response,
        cancel,
        config.max_tokens,
        config.provider_id,
        config.model,
        max_text_bytes,
        measurement,
    )
    .await
}

pub(super) async fn post(
    config: &RequestConfig<'_>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
    request_id: Option<&str>,
    preparation: Option<
        &crate::services::agent_local::context_usage_runtime::PreparedContextAttempt<'_>,
    >,
) -> Result<reqwest::Response, RequestError> {
    let route =
        super::route::resolve(config.provider_id).ok_or(RequestError::InvalidConfiguration)?;
    let prepared = try_build_request_with_evidence(config)?;
    let context_count = prepared.context_count;
    let body = prepared.body;
    if let Some(preparation) = preparation {
        preparation
            .persist_payload(context_count)
            .await
            .map_err(RequestError::Fatal)?;
    }
    super::reasoning_wire::replay::record_evidence(
        config.session_id,
        request_id,
        &prepared.replayed,
    )
    .await;
    crate::services::agent_local::stream_diagnostics_payload::record_provider_payload(
        config.session_id,
        request_id,
        config.provider_id,
        "responses",
        &body,
    )
    .await;
    #[cfg(test)]
    if let Some(response) = super::stream_test_transport::dispatch(config, &body).await {
        return response;
    }
    let client = AuthenticatedClient::new_streaming(
        super::timeouts::connect_timeout(),
        super::timeouts::idle_timeout_for(config.provider_id),
    )
    .map_err(|_| RequestError::InvalidConfiguration)?;
    let url = format!("{}/responses", route.base_url);
    let request_bytes = serde_json::to_vec(&body)
        .map(zeroize::Zeroizing::new)
        .map_err(|_| RequestError::InvalidConfiguration)?
        .len();
    let usage_generation =
        crate::services::provider_usage::credential_generation(config.provider_id);
    let response = super::stream_http_send::send_json_request(
        &client,
        &route,
        &url,
        &body,
        config.purpose,
        config.model,
        config.session_id,
    )
    .await?;
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
    Err(super::stream_http::reject_response(
        response,
        super::stream_http::RejectedResponseContext {
            provider_id: config.provider_id,
            model: config.model,
            error_policy: route.error_policy,
            oauth: route.is_oauth(),
            request_bytes,
            tool_count: config.tools.len(),
            diagnostic: super::provider_diagnostics::ProviderDiagnosticContext::from_payload(
                request_id, &body,
            ),
        },
    )
    .await)
}

fn request_error(error: RequestError) -> String {
    match error {
        RequestError::PayloadTooLarge => "provider_payload_too_large".into(),
        RequestError::InvalidConfiguration => "provider_configuration_invalid".into(),
        RequestError::Fatal(message) => message,
    }
}

#[cfg(test)]
#[path = "openai_responses_tests.rs"]
mod tests;
