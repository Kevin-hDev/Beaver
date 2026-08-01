//! Helpers de parsing/construction pour `openai_compat.rs`.

use super::types::{ChatRequest, ChatResponse, LlmError};
use crate::services::secure_http::{read_bounded, PROVIDER_ERROR_LIMIT};
use reqwest::Response;

pub(super) use super::openai_compat_model_parser::parse_models_list;

/// Construit le payload JSON pour `POST /chat/completions`.
pub fn build_payload(req: &ChatRequest, provider_id: &str, stream: bool) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": req.model,
        "messages": req.messages,
        "stream": stream,
    });
    if let Some(max) = req.max_tokens {
        let field = super::model_metadata::request_output_limit_field(provider_id, &req.model);
        payload[field] = max.into();
    }
    if let Some(t) = req.temperature {
        payload["temperature"] = t.into();
    }
    if !req.tools.is_empty() {
        payload["tools"] = serde_json::to_value(&req.tools).unwrap_or(serde_json::Value::Null);
        payload["tool_choice"] = "auto".into();
    }
    payload
}

/// Parse la réponse de `POST /chat/completions` (non-streaming).
pub fn parse_chat_response(
    body: &serde_json::Value,
    provider_id: &str,
    model: &str,
) -> Result<ChatResponse, LlmError> {
    if let Some(error) = embedded_provider_error(body) {
        return Err(error);
    }
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let usage_value = body.get("usage").or_else(|| body.get("usageMetadata"));
    let usage = usage_value
        .and_then(|value| {
            crate::services::provider_usage::RequestUsage::from_json_with_context(
                value,
                crate::services::provider_usage::UsageContext::chat(provider_id, model),
            )
        })
        .unwrap_or_default();

    Ok(ChatResponse { content, usage })
}

fn embedded_provider_error(body: &serde_json::Value) -> Option<LlmError> {
    let has_error = body.get("error").is_some()
        || body
            .pointer("/choices/0/finish_reason")
            .and_then(serde_json::Value::as_str)
            == Some("error");
    if !has_error {
        return None;
    }
    let status = body
        .pointer("/error/code")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u16::try_from(value).ok());
    let code = match status {
        Some(429) => super::provider_error::ProviderErrorCode::RateLimited,
        Some(408 | 500 | 502 | 503 | 504) => {
            super::provider_error::ProviderErrorCode::ProviderTemporarilyUnavailable
        }
        _ => super::provider_error::ProviderErrorCode::ProviderRequestRejected,
    };
    Some(LlmError::KnownProvider(code))
}

/// Mappe un statut HTTP d'erreur vers un `LlmError` approprié.
/// Le body fournisseur est lu de façon bornée puis remplacé par un code sûr dans les logs.
pub async fn map_error_status(resp: Response, provider_id: &str) -> LlmError {
    let status = resp.status().as_u16();
    match status {
        401 | 403 => LlmError::Unauthorized,
        429 => {
            let retry_after_secs = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            LlmError::RateLimit { retry_after_secs }
        }
        _ => {
            let body = zeroize::Zeroizing::new(
                read_bounded(resp, PROVIDER_ERROR_LIMIT)
                    .await
                    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                    .unwrap_or_default(),
            );
            let code = super::provider_error::classify_http(provider_id, status, &body);
            let log_code = super::provider_error::safe_log_code(provider_id, status, &body);
            eprintln!("[llm] HTTP {status} code={log_code}");
            if status == 402 {
                return LlmError::KnownProvider(code);
            }
            LlmError::Http {
                status,
                message: "erreur serveur provider".into(),
            }
        }
    }
}
