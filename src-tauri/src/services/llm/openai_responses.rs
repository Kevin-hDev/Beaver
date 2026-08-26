use crate::services::agent_local::types_ollama::StreamOutcome;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::secure_http::{read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT};
use tokio_util::sync::CancellationToken;

use super::stream_http::{RequestConfig, RequestError};

#[cfg(test)]
pub(super) fn build_request(config: &RequestConfig<'_>) -> serde_json::Value {
    try_build_request(config).expect("a request without a continuation target cannot be rejected")
}

pub(super) fn try_build_request(
    config: &RequestConfig<'_>,
) -> Result<serde_json::Value, RequestError> {
    let (instructions, input) =
        crate::services::codex_client::convert::convert_messages_with_tools_and_continuity(
            config.messages,
            config.tools,
            config.continuation_target,
        )
        .map_err(|_| RequestError::InvalidConfiguration)?;
    let mut body = serde_json::json!({
        "model": config.model,
        "instructions": instructions,
        "input": input,
        "stream": true,
        "store": false,
        "tools": crate::services::codex_client::convert::convert_tools_to_responses_api(
            config.provider_id,
            config.model,
            config.tools,
        ),
        "tool_choice": "auto",
        "parallel_tool_calls": false,
        "prompt_cache_key": super::prompt_cache_policy::routing_key(
            config.provider_id,
            config.model,
            config.session_id,
        ),
        "include": ["reasoning.encrypted_content"],
    });
    if let Some(tier) = config.fast_mode.api_value() {
        body["service_tier"] = tier.into();
    }
    if let Some(limit) = config.max_tokens {
        body["max_output_tokens"] = limit.into();
    }
    if let Some(effort) = requested_effort(config) {
        body["reasoning"] = if effort == "none" {
            serde_json::json!({"effort": effort})
        } else {
            serde_json::json!({"effort": effort, "summary": "auto"})
        };
    }
    Ok(body)
}

fn requested_effort(config: &RequestConfig<'_>) -> Option<&'static str> {
    if config.provider_id == "xai" {
        return crate::services::llm::providers::xai::reasoning_effort(
            config.model,
            config.reasoning_mode,
        );
    }
    if config.think || config.reasoning_mode == Some("off") {
        return crate::services::reasoning::openai_effort(config.reasoning_mode);
    }
    None
}

pub(super) async fn stream_chat(
    on_event: &crate::services::agent_local::stream_events::AgentEventEmitter,
    config: &RequestConfig<'_>,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    reasoning_capture: Option<super::reasoning_wire::ReasoningCapture>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    let response = post(config, measurement.as_deref_mut())
        .await
        .map_err(request_error)?;
    crate::services::codex_client::stream::consume_external_responses_sse(
        on_event,
        response,
        cancel,
        buffer_content,
        realtime_budget,
        config.provider_id,
        config.model,
        config.tools,
        reasoning_capture,
        measurement,
    )
    .await
}

pub(super) async fn collect_silent(
    config: &RequestConfig<'_>,
    cancel: CancellationToken,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    let response = post(config, measurement.as_deref_mut())
        .await
        .map_err(request_error)?;
    crate::services::codex_client::stream_silent::consume_external_responses_sse_silent(
        response,
        cancel,
        config.max_tokens,
        config.provider_id,
        config.model,
        measurement,
    )
    .await
}

pub(super) async fn post(
    config: &RequestConfig<'_>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<reqwest::Response, RequestError> {
    let route =
        super::route::resolve(config.provider_id).ok_or(RequestError::InvalidConfiguration)?;
    let body = try_build_request(config)?;
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
    let status = response.status();
    let has_retry_after = response.headers().contains_key("retry-after");
    let error_body = read_bounded(response, PROVIDER_ERROR_LIMIT)
        .await
        .map(|bytes| zeroize::Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()))
        .unwrap_or_default();
    let details = super::provider_error::safe_details(&error_body);
    super::provider_diagnostics::record_http_failure(
        config.provider_id,
        config.model,
        status.as_u16(),
        details,
        request_bytes,
        config.tools.len(),
    );
    let log_code =
        super::provider_error::safe_log_code(config.provider_id, status.as_u16(), &error_body);
    ::log::warn!(
        "[{} responses] HTTP {status} code={log_code}",
        config.provider_id
    );
    Err(super::stream_http::classify_error(
        status.as_u16(),
        &error_body,
        route.display_name,
        route.chat_provider_id,
        false,
        has_retry_after,
    ))
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
