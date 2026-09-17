use crate::services::agent_local::ollama_client::OllamaClient;
use crate::services::agent_local::ollama_tool_role::wrap_tool_results;
use crate::services::agent_local::ollama_wire;
use crate::services::agent_local::types_ollama::ChatMessage;
use std::time::Duration;

pub async fn collect_chat_with_timeout_and_limit_global(
    model: &str,
    messages: Vec<ChatMessage>,
    timeout: Duration,
    num_predict: Option<u32>,
) -> Result<(String, u32), String> {
    let client = OllamaClient::from_global()?;
    collect_chat_with_timeout_and_limit(&client, model, messages, timeout, num_predict).await
}

pub async fn collect_chat_with_timeout_and_limit(
    ollama: &OllamaClient,
    model: &str,
    messages: Vec<ChatMessage>,
    timeout: Duration,
    num_predict: Option<u32>,
) -> Result<(String, u32), String> {
    // Conversion `role:"tool"` → `role:"user"` + `<tool_response>` (cf. ollama_tool_role).
    let placement = crate::services::llm::route_profile::payload_policy("ollama", model)
        .expect("Ollama route profile")
        .message
        .tool_results;
    let wire_messages = wrap_tool_results(&messages, placement);
    let mut body = serde_json::json!({
        "model": model,
        "messages": ollama_wire::messages_value(&wire_messages),
        "stream": false,
        "truncate": false,
    });
    if let Some(limit) = num_predict {
        body["options"] = serde_json::json!({
            "temperature": 0.2,
            "num_predict": limit,
        });
    }

    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| format!("Client HTTP : {e}"))?;

    let base_url = ollama.base_url().await?;
    let resp = client
        .post(format!("{base_url}/api/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                "ollama_connection_lost".to_string()
            } else {
                format!("Ollama: {e}")
            }
        })?;

    if !resp.status().is_success() {
        return Err(format!("Ollama HTTP {}", resp.status()));
    }

    let value: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Réponse Ollama invalide : {e}"))?;

    let content = value["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let tokens = value["eval_count"].as_u64().unwrap_or(0) as u32;
    Ok((content, tokens))
}

pub(crate) async fn collect_extension_text(
    model: &str,
    prompt: &str,
    timeout: Duration,
    num_predict: u32,
) -> Result<(String, u32, Option<String>), String> {
    let ollama = OllamaClient::from_global()?;
    let base_url = ollama.base_url().await?;
    let client = crate::services::secure_http::AuthenticatedClient::new_loopback(timeout)
        .map_err(|_| "ollama-runtime-error".to_string())?;
    let response = client
        .send_success(
            client
                .post(format!("{base_url}/api/chat"))
                .json(&serde_json::json!({
                    "model": model,
                    "messages": [{"role": "user", "content": prompt}],
                    "stream": false,
                    "truncate": false,
                    "options": {"temperature": 0.2, "num_predict": num_predict},
                })),
        )
        .await
        .map_err(|_| "ollama_connection_lost".to_string())?;
    let value: serde_json::Value = crate::services::secure_http::read_json_bounded(
        response,
        crate::services::extensions::types::MAX_MODEL_RESULT_BYTES,
    )
    .await
    .map_err(|_| "provider_payload_too_large".to_string())?;
    let content = value
        .pointer("/message/content")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "provider_connection_failed".to_string())?;
    if content.len() > crate::services::extensions::types::MAX_MODEL_RESULT_BYTES {
        return Err("provider_payload_too_large".to_string());
    }
    let eval_count = value
        .get("eval_count")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| value.try_into().ok())
        .unwrap_or(0);
    let done_reason = value
        .get("done_reason")
        .and_then(serde_json::Value::as_str)
        .filter(|reason| reason.len() <= 64)
        .map(str::to_string);
    Ok((content.to_string(), eval_count, done_reason))
}

pub(crate) async fn list_extension_models(timeout: Duration) -> Result<Vec<String>, String> {
    let ollama = OllamaClient::from_global()?;
    let base_url = ollama.base_url().await?;
    let client = crate::services::secure_http::AuthenticatedClient::new_loopback(timeout)
        .map_err(|_| "ollama-runtime-error".to_string())?;
    let response = client
        .send_success(client.get(format!("{base_url}/api/tags")))
        .await
        .map_err(|_| "ollama_connection_lost".to_string())?;
    let value: serde_json::Value = crate::services::secure_http::read_json_bounded(
        response,
        crate::services::secure_http::CODEX_MODELS_BODY_LIMIT,
    )
    .await
    .map_err(|_| "model_catalog_unavailable".to_string())?;
    let mut models = value
        .get("models")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "model_catalog_unavailable".to_string())?
        .iter()
        .take(500)
        .filter_map(|model| model.get("name").and_then(serde_json::Value::as_str))
        .filter(|name| crate::services::llm::runtime_models::valid_model_id(name))
        .map(str::to_string)
        .collect::<Vec<_>>();
    models.sort();
    models.dedup();
    Ok(models)
}
