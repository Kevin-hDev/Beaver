use std::future::Future;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use super::types::{CodexRequest, ReasoningConfig};
use super::{convert, request_http};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::provider_error::ProviderErrorCode;

pub async fn post_codex_stream(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    cancel: &CancellationToken,
) -> Result<reqwest::Response, String> {
    send_request(
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        request_http::RequestDeadline::Streaming,
        cancel,
    )
    .await
}

pub async fn post_codex_stream_with_timeout(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    timeout: Duration,
    cancel: &CancellationToken,
) -> Result<reqwest::Response, String> {
    send_request(
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        request_http::RequestDeadline::Total(timeout),
        cancel,
    )
    .await
}

async fn send_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    _think: bool,
    reasoning_mode: Option<&str>,
    deadline: request_http::RequestDeadline,
    cancel: &CancellationToken,
) -> Result<reqwest::Response, String> {
    let body = build_codex_request(model, messages, tools, reasoning_mode);
    let body_json = serde_json::to_string(&body)
        .map_err(|_| provider_error(ProviderErrorCode::ProviderConfigurationInvalid))?;
    cancel_aware(
        cancel,
        request_http::post(&body_json, model, tools.len(), deadline),
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
) -> CodexRequest {
    let (instructions, input) = convert::convert_messages_with_tools(messages, tools);
    let converted_tools = convert::convert_tools_to_responses_api(tools);
    CodexRequest {
        model: model.to_string(),
        instructions,
        input,
        stream: true,
        store: false,
        tools: converted_tools,
        tool_choice: "auto".to_string(),
        parallel_tool_calls: false,
        reasoning: Some(ReasoningConfig {
            effort: crate::services::reasoning::codex_effort(model, reasoning_mode),
            summary: "auto".to_string(),
        }),
        include: vec!["reasoning.encrypted_content".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_request_keeps_only_model_supported_effort() {
        let sol = build_codex_request("gpt-5.6-sol", &[], &[], Some("ultra"));
        let luna = build_codex_request("gpt-5.6-luna", &[], &[], Some("ultra"));

        assert_eq!(sol.reasoning.unwrap().effort, "ultra");
        assert_eq!(luna.reasoning.unwrap().effort, "medium");
    }

    #[test]
    fn request_keeps_the_official_empty_tools_contract() {
        let request = build_codex_request("gpt-5.6-sol", &[], &[], None);
        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["tools"], serde_json::json!([]));
        assert_eq!(json["tool_choice"], "auto");
    }

    #[tokio::test]
    async fn cancellation_is_observed_before_response_headers() {
        let cancel = CancellationToken::new();
        cancel.cancel();

        let result = cancel_aware(&cancel, std::future::pending::<Result<(), String>>()).await;

        assert_eq!(result.unwrap_err(), "Annulé");
    }
}
