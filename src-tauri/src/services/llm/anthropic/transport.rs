use crate::services::agent_local::types_ollama::{StreamOutcome, StreamResult};
use crate::services::llm::stream_http::{RequestConfig, RequestError};
use crate::services::secure_http::AuthenticatedClient;
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
    preparation: Option<
        &crate::services::agent_local::context_usage_runtime::PreparedContextAttempt<'_>,
    >,
) -> Result<StreamOutcome, String> {
    let response = post(
        config,
        measurement.as_deref_mut(),
        Some(request_id),
        preparation,
    )
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
    let response = post(config, measurement.as_deref_mut(), None, None)
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
    preparation: Option<
        &crate::services::agent_local::context_usage_runtime::PreparedContextAttempt<'_>,
    >,
) -> Result<reqwest::Response, RequestError> {
    let route = crate::services::llm::route::resolve(config.provider_id)
        .ok_or(RequestError::InvalidConfiguration)?;
    let estimated = crate::services::compress::token_estimate::estimate_request_tokens_for_provider(
        config.provider_id,
        config.messages,
        config.tools,
    );
    let requested_max_tokens = {
        #[cfg(debug_assertions)]
        {
            crate::services::reasoning_fixture_budget::output_limit(config.max_tokens)
        }
        #[cfg(not(debug_assertions))]
        {
            config.max_tokens
        }
    };
    let max_tokens = crate::services::llm::stream_max_tokens::resolve(
        route.canonical_provider_id,
        config.model,
        requested_max_tokens,
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
    let context_count = prepared.context_count;
    if let Some(preparation) = preparation {
        preparation
            .persist_payload(context_count)
            .await
            .map_err(RequestError::Fatal)?;
    }
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
        .send_generation_authenticated(
            &client,
            config.purpose,
            &prepared.payload,
            |token, inherited| {
                let request = client.post(&url).headers(inherited).json(&prepared.payload);
                let request = crate::services::llm::request_auth::apply(request, header, token);
                static_headers
                    .iter()
                    .fold(request, |request, (name, value)| {
                        request.header(*name, *value)
                    })
            },
        )
        .await
        .map_err(super::transport_error::map_route_error)?;
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
    super::transport_error::classify_response(
        response,
        &route,
        config,
        request_bytes,
        request_id,
        &prepared.payload,
    )
    .await
}
