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
