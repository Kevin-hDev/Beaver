use super::provider_error::ProviderErrorCode;
use super::stream_convert::messages_to_openai;
pub(super) use super::stream_http_error::RequestError;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::route::{self, LlmRoute, RouteError};
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

async fn send_json_request(
    client: &AuthenticatedClient,
    route: &LlmRoute,
    url: &str,
    payload: &serde_json::Value,
    purpose: RequestPurpose,
) -> Result<reqwest::Response, RequestError> {
    route
        .send_authenticated(client, purpose, |token, headers| {
            client
                .post(url)
                .headers(headers)
                .bearer_auth(token)
                .json(payload)
        })
        .await
        .map_err(|error| match error {
            RouteError::Unauthorized if route.is_oauth() => {
                RequestError::Fatal("oauth_reauthentication_required".into())
            }
            RouteError::Unauthorized => RequestError::Fatal("auth_failed".into()),
            RouteError::Forbidden => RequestError::Fatal("provider_access_unavailable".into()),
            RouteError::Network => {
                RequestError::Fatal(ProviderErrorCode::ProviderConnectionFailed.as_str().into())
            }
        })
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
    let payload = build_chat_payload(cfg, &route);

    let client = AuthenticatedClient::new(timeout).map_err(|_| {
        RequestError::Fatal(
            ProviderErrorCode::ProviderConfigurationInvalid
                .as_str()
                .into(),
        )
    })?;
    let usage_generation =
        crate::services::provider_usage::credential_generation(route.chat_provider_id);
    let resp = send_json_request(&client, &route, &url, &payload, cfg.purpose).await?;

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

fn build_chat_payload(cfg: &RequestConfig<'_>, route: &LlmRoute) -> serde_json::Value {
    let provider_id = route.canonical_provider_id;
    let mut payload = serde_json::json!({
        "model": cfg.model,
        "messages": messages_to_openai(cfg.messages, provider_id),
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    if let Some(max) = cfg.max_tokens.or(route.default_max_tokens) {
        let field = if matches!(provider_id, "openai" | "openrouter")
            && super::providers::openai::is_gpt_56(cfg.model)
        {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
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
    provider_name: &str,
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
        413 => RequestError::Fatal("Requête trop volumineuse (limite TPM dépassée)".into()),
        429 => RequestError::Fatal("rate_limit".into()),
        400 if body.contains("Developer instruction") || body.contains("system_instruction") => {
            RequestError::Fatal("Ce modèle ne supporte pas les instructions système via ce provider. Essaie un autre modèle.".into())
        }
        400 if body.contains("must be a string") => {
            RequestError::RetryWithoutImages("Format image non supporté par ce provider".into())
        }
        404 if body.contains("tool use") || body.contains("tool_use") => {
            RequestError::RetryWithoutTools("Aucun endpoint ne supporte les tools pour ce modèle".into())
        }
        404 if body.contains("image") => {
            RequestError::RetryWithoutImages("Aucun endpoint ne supporte les images pour ce modèle".into())
        }
        400 if body.contains("image") => {
            RequestError::RetryWithoutImages("Ce modèle ne supporte pas les images".into())
        }
        _ => RequestError::Fatal(format!("{provider_name} HTTP {status}")),
    }
}

#[cfg(test)]
#[path = "stream_http_tests.rs"]
mod tests;
