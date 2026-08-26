#![expect(
    clippy::too_many_arguments,
    reason = "transport boundary keeps the captured Fast mode explicit"
)]
use std::future::Future;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use super::types::{CodexRequest, ReasoningConfig};
use super::{convert, request_http};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::llm::provider_error::ProviderErrorCode;

#[cfg(test)]
pub async fn post_codex_stream(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
    cancel: &CancellationToken,
) -> Result<reqwest::Response, String> {
    post_codex_stream_with_continuity(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        cancel,
        None,
    )
    .await
}

pub async fn post_codex_stream_with_continuity(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
    cancel: &CancellationToken,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<reqwest::Response, String> {
    send_request(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        request_http::RequestDeadline::Streaming,
        cancel,
        continuation_target,
    )
    .await
}

pub async fn post_codex_stream_with_timeout(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
    timeout: Duration,
    cancel: &CancellationToken,
) -> Result<reqwest::Response, String> {
    send_request(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        request_http::RequestDeadline::Total(timeout),
        cancel,
        None,
    )
    .await
}

async fn send_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
    deadline: request_http::RequestDeadline,
    cancel: &CancellationToken,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<reqwest::Response, String> {
    let body = build_codex_request_with_continuity(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        continuation_target,
    )?;
    let routing_hint = super::routing_hint::for_request(&body)?;
    let body_json = serde_json::to_string(&body)
        .map_err(|_| provider_error(ProviderErrorCode::ProviderConfigurationInvalid))?;
    cancel_aware(
        cancel,
        request_http::post(&body_json, &routing_hint, model, tools.len(), deadline),
    )
    .await
}

async fn cancel_aware<T, F>(cancel: &CancellationToken, future: F) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err("Annulé".to_string()),
        result = future => result,
    }
}

fn provider_error(code: ProviderErrorCode) -> String {
    code.as_str().to_string()
}

pub(super) fn build_codex_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
) -> CodexRequest {
    build_codex_request_with_continuity(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        None,
    )
    .expect("a request without a continuation target cannot be rejected")
}

pub(super) fn build_codex_request_with_continuity(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<CodexRequest, String> {
    let (instructions, input) =
        convert::convert_messages_with_tools_and_continuity(messages, tools, continuation_target)
            .map_err(|_| "reasoning_continuity_invalid".to_string())?;
    let converted_tools = convert::convert_tools_to_responses_api(super::PROVIDER_ID, model, tools);
    Ok(CodexRequest {
        model: model.to_string(),
        instructions,
        input,
        stream: true,
        store: false,
        tools: converted_tools,
        tool_choice: "auto".to_string(),
        parallel_tool_calls: false,
        prompt_cache_key: crate::services::llm::prompt_cache_policy::routing_key(
            super::PROVIDER_ID,
            model,
            session_id,
        ),
        reasoning: Some(ReasoningConfig {
            effort: crate::services::reasoning::codex_effort(model, reasoning_mode),
            summary: "auto".to_string(),
        }),
        service_tier: fast_mode.codex_value().map(str::to_string),
        include: vec!["reasoning.encrypted_content".to_string()],
    })
}

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;
