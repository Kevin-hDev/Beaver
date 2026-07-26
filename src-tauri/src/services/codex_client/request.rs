use std::time::Duration;

use super::types::{CodexRequest, ReasoningConfig, CODEX_API_BASE};
use super::{convert, http_error};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::codex_oauth::store::CodexTokens;
use crate::services::codex_oauth::token;
use crate::services::llm::provider_error::ProviderErrorCode;
use crate::services::secure_http::{AuthenticatedClient, SecureHttpError};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);

struct RequestOptions {
    timeout: Duration,
}

pub async fn post_codex_stream(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
) -> Result<reqwest::Response, String> {
    let creds = token::ensure_valid().await?;
    send_request(
        &creds,
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        RequestOptions {
            timeout: REQUEST_TIMEOUT,
        },
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
    let creds = token::ensure_valid().await?;
    send_request(
        &creds,
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        RequestOptions { timeout },
    )
    .await
}

async fn send_request(
    creds: &CodexTokens,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    _think: bool,
    reasoning_mode: Option<&str>,
    options: RequestOptions,
) -> Result<reqwest::Response, String> {
    let body = build_codex_request(model, messages, tools, reasoning_mode);
    let body_json = serde_json::to_string(&body)
        .map_err(|_| provider_error(ProviderErrorCode::ProviderConfigurationInvalid))?;
    let client = AuthenticatedClient::new(options.timeout)
        .map_err(|error| secure_http_error(error).to_string())?;
    let request = client
        .post(format!("{CODEX_API_BASE}/responses"))
        .bearer_auth(creds.access.as_str())
        .header("chatgpt-account-id", creds.account_hint.as_str())
        .header("OpenAI-Beta", "responses=experimental")
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .body(body_json);
    let response = client
        .send(request)
        .await
        .map_err(|error| secure_http_error(error).to_string())?;
    http_error::require_success(response).await
}

fn secure_http_error(error: SecureHttpError) -> &'static str {
    match error {
        SecureHttpError::Configuration => ProviderErrorCode::ProviderConfigurationInvalid.as_str(),
        SecureHttpError::Status => ProviderErrorCode::ProviderRequestRejected.as_str(),
        _ => ProviderErrorCode::ProviderConnectionFailed.as_str(),
    }
}

fn provider_error(code: ProviderErrorCode) -> String {
    code.as_str().to_string()
}

fn build_codex_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
) -> CodexRequest {
    let (instructions, input) = convert::convert_messages(messages);
    let converted_tools = convert::convert_tools_to_responses_api(tools);
    CodexRequest {
        model: model.to_string(),
        instructions,
        input,
        stream: true,
        store: false,
        tools: converted_tools,
        tool_choice: if tools.is_empty() {
            None
        } else {
            Some("auto".to_string())
        },
        reasoning: Some(ReasoningConfig {
            effort: crate::services::reasoning::codex_effort(model, reasoning_mode),
            summary: "auto".to_string(),
        }),
        include: Some(vec!["reasoning.encrypted_content".to_string()]),
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
    fn codex_transport_errors_use_stable_codes() {
        assert_eq!(
            secure_http_error(SecureHttpError::Configuration),
            "provider_configuration_invalid"
        );
        assert_eq!(
            secure_http_error(SecureHttpError::Status),
            "provider_request_rejected"
        );
        assert_eq!(
            secure_http_error(SecureHttpError::Request),
            "provider_connection_failed"
        );
    }
}
