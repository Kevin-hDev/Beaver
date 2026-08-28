#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use super::stream_consume::consume_stream;
use super::stream_http::{RequestConfig, RequestError};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm::request_purpose::RequestPurpose;
use tokio_util::sync::CancellationToken;
pub async fn stream_chat_no_done(
    on_event: &AgentEventEmitter,
    session_id: &str,
    request_id: &str,
    turn: u32,
    attempt: u32,
    provider_id: &str,
    fast_mode: super::fast_mode::FastModeRequest,
    purpose: RequestPurpose,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    cancel: CancellationToken,
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
    reasoning_capture: Option<super::reasoning_wire::ReasoningCapture>,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<StreamOutcome, String> {
    let transport = super::stream_dispatch::resolve_transport(
        provider_id,
        model,
        super::stream_dispatch::InvocationKind::Interactive,
        purpose,
    )
    .await
    .map_err(super::stream_dispatch::RouteSelectionError::code)?;
    let mut measurement = super::stream_metrics::start(
        &transport,
        provider_id,
        model,
        Some(session_id),
        request_id,
        Some(turn),
        attempt,
        crate::services::provider_usage::UsageWorkload::Primary,
        fast_mode,
    );
    let result = match transport.client {
        super::stream_dispatch::ClientKind::Codex => {
            crate::services::codex_client::stream::stream_chat_with_budget(
                on_event,
                session_id,
                request_id,
                model,
                messages,
                tools,
                reasoning_mode,
                fast_mode,
                cancel,
                buffer_content,
                realtime_budget,
                reasoning_capture,
                continuation_target,
                measurement.as_mut(),
            )
            .await
        }
        super::stream_dispatch::ClientKind::Responses => {
            let config = RequestConfig {
                provider_id,
                model,
                messages,
                tools,
                think,
                reasoning_mode,
                max_tokens: None,
                purpose,
                session_id: Some(session_id),
                fast_mode,
                continuation_target,
            };
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
                    request: RequestConfig {
                        provider_id,
                        model,
                        messages,
                        tools,
                        think: true,
                        reasoning_mode,
                        max_tokens: None,
                        purpose,
                        session_id: Some(session_id),
                        fast_mode,
                        continuation_target,
                    },
                    cancel,
                    buffer_content,
                    realtime_budget,
                    reasoning_capture,
                    request_id,
                },
                catalog_model,
                measurement.as_mut(),
            )
            .await
        }
        super::stream_dispatch::ClientKind::ChatCompletions => {
            let cfg = RequestConfig {
                provider_id,
                model,
                messages,
                tools,
                think,
                reasoning_mode,
                max_tokens: None,
                purpose,
                session_id: Some(session_id),
                fast_mode,
                continuation_target,
            };
            match super::stream_http::post_chat_request_measured(
                &cfg,
                measurement.as_mut(),
                Some(request_id),
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
                        tools,
                        transport.usage_context(model),
                        transport.fragment_mode,
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
