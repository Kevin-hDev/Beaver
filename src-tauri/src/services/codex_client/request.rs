use std::time::Duration;

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
) -> Result<reqwest::Response, String> {
    send_request(
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        request_http::RequestDeadline::Streaming,
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
) -> Result<reqwest::Response, String> {
    send_request(
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        request_http::RequestDeadline::Total(timeout),
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
) -> Result<reqwest::Response, String> {
    let body = build_codex_request(model, messages, tools, reasoning_mode);
    let body_json = serde_json::to_string(&body)
        .map_err(|_| provider_error(ProviderErrorCode::ProviderConfigurationInvalid))?;
    request_http::post(&body_json, model, tools.len(), deadline).await
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
}
