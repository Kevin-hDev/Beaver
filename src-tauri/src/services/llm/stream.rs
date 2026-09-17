use super::stream_consume::consume_stream;
use super::stream_http::{RequestConfig, RequestError};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamOutcome;
use crate::services::compress::realtime_budget::RealtimeBudget;
use tokio_util::sync::CancellationToken;

pub struct InteractiveStreamRequest<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub request_id: &'a str,
    pub turn: u32,
    pub request: RequestConfig<'a>,
    pub cancel: CancellationToken,
    pub buffer_content: bool,
    pub realtime_budget: Option<RealtimeBudget>,
    pub preparation:
        Option<&'a crate::services::agent_local::context_usage_runtime::PreparedContextAttempt<'a>>,
}

pub async fn stream_chat_no_done(
    context: InteractiveStreamRequest<'_>,
    attempt: u32,
    reasoning_capture: Option<super::reasoning_wire::ReasoningCapture>,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<StreamOutcome, String> {
    let InteractiveStreamRequest {
        on_event,
        request_id,
        turn,
        request,
        cancel,
        buffer_content,
        realtime_budget,
        preparation,
    } = context;
    let session_id = request
        .session_id
        .ok_or_else(|| "provider_configuration_invalid".to_string())?;
    #[cfg(debug_assertions)]
    let transport = if continuation_target.is_some_and(|target| target.is_fixture_candidate()) {
        super::stream_dispatch::resolve_fixture_transport(
            request.provider_id,
            request.model,
            continuation_target.expect("fixture target"),
            request.purpose,
        )
        .await
    } else {
        super::stream_dispatch::resolve_transport(
            request.provider_id,
            request.model,
            super::stream_dispatch::InvocationKind::Interactive,
            request.purpose,
        )
        .await
    }
    .map_err(super::stream_dispatch::RouteSelectionError::code)?;
    #[cfg(not(debug_assertions))]
    let transport = super::stream_dispatch::resolve_transport(
        request.provider_id,
        request.model,
        super::stream_dispatch::InvocationKind::Interactive,
        request.purpose,
    )
    .await
    .map_err(super::stream_dispatch::RouteSelectionError::code)?;
    let mut measurement = super::stream_metrics::start(
        &transport,
        request.provider_id,
        request.model,
        Some(session_id),
        request_id,
        Some(turn),
        attempt,
        crate::services::provider_usage::UsageWorkload::Primary,
        request.fast_mode,
    );
    // Every HTTP family receives the same bounded continuation state; the
    // route profile alone decides whether preview bytes enter its payload.
    let request_config = |request_think| RequestConfig {
        provider_id: request.provider_id,
        model: request.model,
        messages: request.messages,
        tools: request.tools,
        think: request_think,
        reasoning_mode: request.reasoning_mode,
        max_tokens: request.max_tokens,
        purpose: request.purpose,
        session_id: Some(session_id),
        fast_mode: request.fast_mode,
        tool_result_previews: request.tool_result_previews,
        continuation_target,
    };
    let result = match transport.client {
        super::stream_dispatch::ClientKind::Anthropic => {
            let config = request_config(request.think);
            super::anthropic::stream_chat(
                on_event,
                &config,
                cancel,
                buffer_content,
                realtime_budget,
                reasoning_capture,
                request_id,
                measurement.as_mut(),
                preparation,
            )
            .await
        }
        super::stream_dispatch::ClientKind::Codex => {
            crate::services::codex_client::stream::stream_chat_with_budget(
                on_event,
                session_id,
                request_id,
                request.model,
                request.messages,
                request.tools,
                request.reasoning_mode,
                request.fast_mode,
                cancel,
                buffer_content,
                realtime_budget,
                reasoning_capture,
                continuation_target,
                measurement.as_mut(),
                preparation,
            )
            .await
        }
        super::stream_dispatch::ClientKind::Responses => {
            let config = request_config(request.think);
            // Les API publiques OpenAI et xAI utilisent Responses avec leur propre authentification.
            super::openai_responses::stream_chat(
                on_event,
                &config,
                cancel,
                super::openai_responses::ResponseStreamOptions {
                    buffer_content,
                    realtime_budget,
                    reasoning_capture,
                    request_id,
                    preparation,
                },
                measurement.as_mut(),
            )
            .await
        }
        super::stream_dispatch::ClientKind::XaiOauth(_) => {
            let catalog_model = transport
                .xai_catalog_model
                .as_ref()
                .ok_or_else(|| "provider_configuration_invalid".to_string())?;
            super::xai_oauth_transport::stream_chat(
                super::xai_oauth_transport::StreamContext {
                    on_event,
                    request: request_config(true),
                    cancel,
                    buffer_content,
                    realtime_budget,
                    reasoning_capture,
                    request_id,
                },
                catalog_model,
                measurement.as_mut(),
                preparation,
            )
            .await
        }
        super::stream_dispatch::ClientKind::ChatCompletions => {
            let cfg = request_config(request.think);
            match super::stream_http::post_chat_request_measured(
                &cfg,
                measurement.as_mut(),
                Some(request_id),
                preparation,
            )
            .await
            {
                Ok(resp) => {
                    consume_stream(
                        on_event,
                        resp,
                        cancel,
                        buffer_content,
                        realtime_budget,
                        request.tools,
                        transport.usage_context(request.model),
                        transport.fragment_mode,
                        transport.error_policy,
                        reasoning_capture,
                        measurement.as_mut(),
                    )
                    .await
                }
                Err(RequestError::PayloadTooLarge) => Err("provider_payload_too_large".to_string()),
                Err(RequestError::InvalidConfiguration) => Err(
                    super::provider_error::ProviderErrorCode::ProviderConfigurationInvalid
                        .as_str()
                        .to_string(),
                ),
                Err(RequestError::Fatal(msg)) => Err(msg),
            }
        }
        super::stream_dispatch::ClientKind::OllamaLocal => {
            Err("provider_configuration_invalid".to_string())
        }
    };
    super::stream_metrics::finish_stream(measurement, &result).await;
    result
}
pub use super::stream_silent::collect_chat_silent_for_compression;
