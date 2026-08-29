#![expect(
    clippy::too_many_arguments,
    reason = "transport boundary keeps the captured Fast mode explicit"
)]
use std::future::Future;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use super::request_build::build_codex_request_with_continuity_evidence;
use super::request_http;
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
    request_id: Option<&str>,
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
        request_id,
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
    request_id: Option<&str>,
) -> Result<reqwest::Response, String> {
    let prepared = build_codex_request_with_continuity_evidence(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        continuation_target,
    )?;
    crate::services::llm::reasoning_wire::replay::record_evidence(
        session_id,
        request_id,
        &prepared.replayed,
    )
    .await;
    let body = prepared.body;
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

pub(super) use super::request_build::build_codex_request;
#[cfg(test)]
pub(super) use super::request_build::build_codex_request_with_continuity;

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;
