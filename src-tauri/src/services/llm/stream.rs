use super::stream_consume::consume_stream;
use super::stream_http::{post_chat_request, RequestConfig, RequestError};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::vision;
use tokio_util::sync::CancellationToken;
pub async fn stream_chat_no_done(
    on_event: &AgentEventEmitter,
    provider_id: &str,
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
    if provider_id == "codex-oauth" {
        return crate::services::codex_client::stream::stream_chat_with_budget(
            on_event,
            model,
            messages,
            tools,
            think,
            reasoning_mode,
            cancel,
            buffer_content,
            realtime_budget,
        )
        .await;
    }
    let cfg = RequestConfig {
        provider_id,
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        max_tokens: None,
        purpose,
    };
    match post_chat_request(&cfg).await {
        Ok(resp) => {
            let (outcome, _, _) =
                consume_stream(on_event, resp, cancel, buffer_content, realtime_budget).await?;
            Ok(outcome)
        }
        Err(RequestError::RetryWithoutTools(msg)) => {
            eprintln!("[llm stream] retry sans tools: {msg}");
            let cfg2 = RequestConfig {
                provider_id,
                model,
                messages,
                tools: &[],
                think,
                reasoning_mode,
                max_tokens: None,
                purpose,
            };
            let resp = post_chat_request(&cfg2).await.map_err(|e| e.to_string())?;
            let (outcome, _, _) =
                consume_stream(on_event, resp, cancel, buffer_content, realtime_budget).await?;
            Ok(outcome)
        }
        Err(RequestError::RetryWithoutImages(msg)) => {
            eprintln!("[llm stream] retry sans images: {msg}");
            let mut msgs_clean = messages.to_vec();
            if vision::strip_images(&mut msgs_clean) > 0 {
                let _ = on_event.send(StreamEvent::Notice {
                    message_key: vision::NOTICE_UNSUPPORTED_MODEL.to_string(),
                });
            }
            let cfg2 = RequestConfig {
                provider_id,
                model,
                messages: &msgs_clean,
                tools,
                think,
                reasoning_mode,
                max_tokens: None,
                purpose,
            };
            let resp = post_chat_request(&cfg2).await.map_err(|e| e.to_string())?;
            let (outcome, _, _) =
                consume_stream(on_event, resp, cancel, buffer_content, realtime_budget).await?;
            Ok(outcome)
        }
        Err(RequestError::Fatal(msg)) => Err(msg),
    }
}
pub use super::stream_silent::{collect_chat_silent, collect_chat_silent_for_compression};
