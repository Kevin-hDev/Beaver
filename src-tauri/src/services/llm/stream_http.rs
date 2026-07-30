use super::provider_error::ProviderErrorCode;
pub(super) use super::stream_http_error::RequestError;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::route::{self, LlmRoute};
use crate::services::secure_http::{read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT};
pub struct RequestConfig<'a> {
    pub provider_id: &'a str,
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    pub tools: &'a [serde_json::Value],
    pub think: bool,
    pub reasoning_mode: Option<&'a str>,
    pub max_tokens: Option<u32>,
    pub purpose: RequestPurpose,
}

async fn read_provider_error(response: reqwest::Response) -> zeroize::Zeroizing<String> {
    match read_bounded(response, PROVIDER_ERROR_LIMIT).await {
        Ok(bytes) => zeroize::Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => zeroize::Zeroizing::new(String::new()),
    }
}

pub async fn post_chat_request(cfg: &RequestConfig<'_>) -> Result<reqwest::Response, RequestError> {
    post_chat_request_with_timeout(cfg, super::timeouts::request_timeout()).await
}

pub async fn post_chat_request_with_timeout(
    cfg: &RequestConfig<'_>,
    timeout: std::time::Duration,
) -> Result<reqwest::Response, RequestError> {
    if cfg.model.len() > 128 {
        return Err(RequestError::Fatal("nom de modèle trop long".into()));
    }
    let route = route::resolve(cfg.provider_id)
        .ok_or_else(|| RequestError::Fatal("Fournisseur inconnu".to_string()))?;
    let url = format!("{}/chat/completions", route.base_url);
    let estimated_input_tokens =
        crate::services::compress::token_estimate::estimate_request_tokens(cfg.messages, cfg.tools);
    let max_tokens = super::stream_max_tokens::resolve(
        route.canonical_provider_id,
        cfg.model,
        cfg.max_tokens,
        route.auto_max_tokens,
        route.fallback_max_tokens,
        estimated_input_tokens,
    )
    .await
    .map_err(|_| RequestError::PayloadTooLarge)?;
    let payload = build_chat_payload(cfg, &route, max_tokens);
    let request_bytes = serde_json::to_vec(&payload)
        .map(zeroize::Zeroizing::new)
        .map_or(0, |bytes| bytes.len());

    let client = AuthenticatedClient::new(timeout).map_err(|_| {
        RequestError::Fatal(
            ProviderErrorCode::ProviderConfigurationInvalid
                .as_str()
                .into(),
        )
    })?;
    let usage_generation =
        crate::services::provider_usage::credential_generation(route.chat_provider_id);
    let resp =
        super::stream_http_send::send_json_request(&client, &route, &url, &payload, cfg.purpose)
            .await?;

    crate::services::provider_usage::capture_headers(
        route.chat_provider_id,
        usage_generation,
        resp.headers(),
    )
    .await;

    let status = resp.status();
    if !status.is_success() {
        let body = read_provider_error(resp).await;
        let log_code =
            super::provider_error::safe_log_code(route.chat_provider_id, status.as_u16(), &body);
        super::provider_diagnostics::record_http_failure(
            route.chat_provider_id,
            cfg.model,
            status.as_u16(),
            super::provider_error::safe_details(&body),
            request_bytes,
            cfg.tools.len(),
        );
        eprintln!("[llm stream] HTTP {status} code={log_code}");
        return Err(classify_error(
            status.as_u16(),
            &body,
            route.display_name,
            route.chat_provider_id,
            route.is_oauth(),
        ));
    }
    Ok(resp)
}

fn build_chat_payload(
    cfg: &RequestConfig<'_>,
    route: &LlmRoute,
    max_tokens: Option<u32>,
) -> serde_json::Value {
    let provider_id = route.canonical_provider_id;
    let mut payload = serde_json::json!({
        "model": cfg.model,
        "messages": super::stream_convert::messages_to_openai_with_tools(
            cfg.messages,
            provider_id,
            cfg.tools,
        ),
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    if let Some(max) = max_tokens {
        let field = super::model_metadata::request_output_limit_field(provider_id, cfg.model);
        payload[field] = max.into();
    }
    super::stream_reasoning::apply(
        &mut payload,
        provider_id,
        cfg.model,
        cfg.think,
        cfg.reasoning_mode,
    );
    if !cfg.tools.is_empty() {
        let tools = super::tool_schema::tools_for_provider(provider_id, cfg.model, cfg.tools);
        payload["tools"] = serde_json::Value::Array(tools);
        payload["tool_choice"] = "auto".into();
        if provider_id == "zai" {
            payload["tool_stream"] = true.into();
        }
    }
    if provider_id == "openrouter" {
        payload["provider"] = serde_json::json!({
            "require_parameters": true,
            "allow_fallbacks": true,
        });
    }
    payload
}

fn classify_error(
    status: u16,
    body: &str,
    _provider_name: &str,
    provider_id: &str,
    oauth: bool,
) -> RequestError {
    match status {
        402 => RequestError::Fatal(
            super::provider_error::classify_http(provider_id, status, body)
                .as_str()
                .to_string(),
        ),
        401 if oauth => RequestError::Fatal("oauth_reauthentication_required".into()),
        403 if oauth => RequestError::Fatal("provider_access_unavailable".into()),
        401 | 403 => RequestError::Fatal("auth_failed".into()),
        413 => RequestError::PayloadTooLarge,
        429 => RequestError::Fatal("rate_limit".into()),
        500..=599 => RequestError::Fatal(
            ProviderErrorCode::ProviderTemporarilyUnavailable
                .as_str()
                .to_string(),
        ),
        _ => RequestError::Fatal(
            ProviderErrorCode::ProviderRequestRejected
                .as_str()
                .to_string(),
        ),
    }
}

#[cfg(test)]
#[path = "stream_http_classification_tests.rs"]
mod classification_tests;
#[cfg(test)]
#[path = "stream_http_tests.rs"]
mod tests;
