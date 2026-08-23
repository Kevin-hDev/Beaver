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
) -> Result<StreamOutcome, String> {
    let mut measurement = super::stream_metrics::start(
        provider_id,
        model,
        Some(session_id),
        request_id,
        Some(turn),
        attempt,
        crate::services::provider_usage::UsageWorkload::Primary,
    );
    let result = if provider_id == "codex-oauth" {
        crate::services::codex_client::stream::stream_chat_with_budget(
            on_event,
            session_id,
            request_id,
            model,
            messages,
            tools,
            reasoning_mode,
            cancel,
            buffer_content,
            realtime_budget,
            measurement.as_mut(),
        )
        .await
    } else if provider_id == "xai-oauth" {
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
                },
                cancel,
                buffer_content,
                realtime_budget,
            },
            measurement.as_mut(),
        )
        .await
    } else {
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
        };
        match super::stream_http::post_chat_request_measured(&cfg, measurement.as_mut()).await {
            Ok(resp) => {
                consume_stream(
                    on_event,
                    resp,
                    cancel,
                    buffer_content,
                    realtime_budget,
                    tools,
                    crate::services::provider_usage::UsageContext::chat(
                        super::route::canonical_provider_id(provider_id),
                        model,
                    ),
                    measurement.as_mut(),
                )
                .await
            }
            Err(RequestError::PayloadTooLarge) => Err("provider_payload_too_large".to_string()),
            Err(RequestError::InvalidConfiguration) => {
                Err("provider_configuration_invalid".to_string())
            }
            Err(RequestError::Fatal(msg)) => Err(msg),
        }
    };
    super::stream_metrics::finish_stream(measurement, &result).await;
    result
}
pub use super::stream_silent::collect_chat_silent_for_compression;
