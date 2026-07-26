//! Helpers de parsing/construction pour `openai_compat.rs`.

use super::types::{ChatRequest, ChatResponse, LlmError, ModelInfo};
use crate::services::secure_http::{read_bounded, PROVIDER_ERROR_LIMIT};
use reqwest::Response;

/// Construit le payload JSON pour `POST /chat/completions`.
pub fn build_payload(req: &ChatRequest, stream: bool) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": req.model,
        "messages": req.messages,
        "stream": stream,
    });
    if let Some(max) = req.max_tokens {
        let field = if super::providers::openai::is_gpt_56(&req.model) {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
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

/// Parse la réponse de `GET /models`.
pub fn parse_models_list(
    body: &serde_json::Value,
    provider_id: &str,
) -> Result<Vec<ModelInfo>, LlmError> {
    let data = body["data"].as_array().ok_or_else(|| {
        LlmError::Parse(format!("champ 'data' absent ou invalide ({})", provider_id))
    })?;

    let models = data
        .iter()
        .take(500)
        .filter_map(|m| {
            let id = m["id"].as_str()?.to_string();
            let owned_by = m["owned_by"].as_str().map(|s| s.to_string());
            let context_length = m["context_length"]
                .as_u64()
                .or_else(|| m["context_window"].as_u64())
                .or_else(|| m["max_context_length"].as_u64())
                .map(|v| v as u32)
                .or_else(|| known_context_length(provider_id, &id));
            let supported_parameters = supported_parameters(m);
            let has_param = |name: &str| supported_parameters.iter().any(|p| p == name);
            // OpenRouter: `supported_parameters` incluant "tools"
            // Mistral: `capabilities.function_calling: true`
            let supports_tools = has_param("tools")
                || m["capabilities"]["function_calling"]
                    .as_bool()
                    .unwrap_or(false)
                || super::tool_capable::supports_tools(provider_id, &id);
            let is_chat = m["capabilities"]["completion_chat"]
                .as_bool()
                .unwrap_or(true);
            if !is_chat && m["capabilities"].is_object() {
                return None;
            }
            let supports_vision = m["capabilities"]["vision"].as_bool().unwrap_or(false)
                || m["architecture"]["modality"]
                    .as_str()
                    .map(|s| s.contains("image->") || s.contains("image+"))
                    .unwrap_or(false)
                || m["architecture"]["input_modalities"]
                    .as_array()
                    .map(|arr| arr.iter().any(|v| v.as_str() == Some("image")))
                    .unwrap_or(false)
                || super::tool_capable::supports_vision(provider_id, &id);
            let is_free = is_price_free(&m["pricing"]["prompt"])
                && is_price_free(&m["pricing"]["completion"]);
            let supports_thinking = has_param("reasoning")
                || has_param("reasoning_effort")
                || has_param("include_reasoning")
                || super::tool_capable::supports_thinking(provider_id, &id);
            let reasoning_modes = if supports_thinking {
                crate::services::reasoning::supported_modes(provider_id, &id, true)
                    .iter()
                    .map(|mode| mode.to_string())
                    .collect()
            } else {
                Vec::new()
            };
            Some(ModelInfo {
                id,
                display_name: None,
                owned_by,
                context_length,
                supports_tools,
                supports_vision,
                supports_thinking,
                reasoning_modes,
                default_reasoning_mode: None,
                is_free,
            })
        })
        .collect();

    Ok(models)
}

fn known_context_length(provider_id: &str, model_id: &str) -> Option<u32> {
    match provider_id {
        "openai" | "openrouter" => super::providers::openai::context_length(model_id),
        _ => None,
    }
}

/// Parse la réponse de `POST /chat/completions` (non-streaming).
pub fn parse_chat_response(body: &serde_json::Value) -> Result<ChatResponse, LlmError> {
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let usage_value = body.get("usage").or_else(|| body.get("usageMetadata"));
    let usage = usage_value
        .and_then(crate::services::provider_usage::RequestUsage::from_json)
        .unwrap_or_default();

    Ok(ChatResponse { content, usage })
}

fn supported_parameters(m: &serde_json::Value) -> Vec<String> {
    m["supported_parameters"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .take(64)
                .collect()
        })
        .unwrap_or_default()
}

/// Prix = "0" ou absent → gratuit.
fn is_price_free(v: &serde_json::Value) -> bool {
    match v.as_str() {
        Some(s) => s == "0" || s == "0.0" || s == "0.00",
        None => v.is_null(),
    }
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
