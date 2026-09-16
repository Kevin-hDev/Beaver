#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::llm::request_purpose::RequestPurpose;
use tokio_util::sync::CancellationToken;

#[cfg(test)]
use super::stream_http::RequestConfig;

pub async fn collect_chat_silent_for_compression(
    provider_id: &str,
    fast_mode: super::fast_mode::FastModeRequest,
    model: &str,
    messages: &[ChatMessage],
    max_tokens: u32,
    purpose: RequestPurpose,
    session_id: &str,
    request_id: Option<&str>,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let request_timeout = crate::services::compress::timeouts::compression_request_timeout();
    let idle_timeout = crate::services::compress::timeouts::compression_idle_timeout();
    let generated_request_id = uuid::Uuid::new_v4().to_string();
    let request_id = request_id.unwrap_or(&generated_request_id);
    let reasoning_mode = internal_reasoning_mode(provider_id, model);
    let fast_mode = reasoning_mode
        .map(|_| super::fast_mode::standard_for_internal(provider_id))
        .unwrap_or(fast_mode);
    super::text_generation::collect(
        provider_id,
        fast_mode,
        model,
        messages,
        max_tokens,
        purpose,
        session_id,
        request_id,
        cancel,
        crate::services::provider_usage::UsageWorkload::Compression,
        request_timeout,
        idle_timeout,
        usize::MAX,
        reasoning_mode,
    )
    .await
    .and_then(super::stream_completion::require_complete)
}

fn internal_reasoning_mode(provider_id: &str, model: &str) -> Option<&'static str> {
    // Independent summaries use the lightest documented effort, never the
    // user's conversational selection or a costly remote default.
    match provider_id {
        "google" if model == "gemini-3.8-flash" => Some("low"),
        "zai" if model == "glm-5.3-flash" => Some("low"),
        "openai" if model == "gpt-6-astra" => Some("low"),
        _ => None,
    }
}

#[cfg(test)]
fn request_config<'a>(
    provider_id: &'a str,
    fast_mode: super::fast_mode::FastModeRequest,
    model: &'a str,
    messages: &'a [ChatMessage],
    max_tokens: Option<u32>,
    purpose: RequestPurpose,
    session_id: Option<&'a str>,
) -> RequestConfig<'a> {
    let reasoning_mode = internal_reasoning_mode(provider_id, model);
    RequestConfig {
        provider_id,
        fast_mode: reasoning_mode
            .map(|_| super::fast_mode::standard_for_internal(provider_id))
            .unwrap_or(fast_mode),
        model,
        messages,
        tools: &[],
        think: reasoning_mode.is_some(),
        reasoning_mode,
        max_tokens,
        purpose,
        session_id,
        tool_result_previews: None,
        continuation_target: None,
    }
}

#[cfg(test)]
#[path = "stream_silent_request_tests.rs"]
mod stream_silent_request_tests;
