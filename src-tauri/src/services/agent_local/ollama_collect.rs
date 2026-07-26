use crate::services::agent_local::ollama_base_url;
use crate::services::agent_local::ollama_tool_role::wrap_tool_results;
use crate::services::agent_local::ollama_wire;
use crate::services::agent_local::types_ollama::ChatMessage;
use std::time::Duration;

const COLLECT_TIMEOUT_SECS: u64 = 180;

/// Appel Ollama non-interactif (sans streaming UI).
pub async fn collect_chat(
    model: &str,
    messages: Vec<ChatMessage>,
) -> Result<(String, u32), String> {
    collect_chat_with_timeout(model, messages, Duration::from_secs(COLLECT_TIMEOUT_SECS)).await
}

pub async fn collect_chat_with_timeout(
    model: &str,
    messages: Vec<ChatMessage>,
    timeout: Duration,
) -> Result<(String, u32), String> {
    collect_chat_with_timeout_and_limit(model, messages, timeout, None).await
}

pub async fn collect_chat_with_timeout_and_limit(
    model: &str,
    messages: Vec<ChatMessage>,
    timeout: Duration,
    num_predict: Option<u32>,
) -> Result<(String, u32), String> {
    // Conversion `role:"tool"` → `role:"user"` + `<tool_response>` (cf. ollama_tool_role).
    let wire_messages = wrap_tool_results(&messages);
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

    let resp = client
        .post(format!("{}/api/chat", ollama_base_url()))
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
