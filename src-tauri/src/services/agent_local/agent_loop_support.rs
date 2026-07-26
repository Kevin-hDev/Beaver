use crate::services::agent_local::ollama_base_url;
use crate::services::agent_local::types_ollama::{
    ChatMessage, ChatRequest, OllamaThink, StreamResult, ToolCallFunction, ToolCallOllama,
};
use std::path::Path;

pub fn build_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: OllamaThink,
) -> ChatRequest {
    let keep_alive = crate::services::config::read_config()
        .map(|c| c.advanced.keep_alive)
        .unwrap_or_else(|_| "5m".to_string());
    let keep_alive = if keep_alive == "forever" {
        "-1m".to_string()
    } else {
        keep_alive
    };

    ChatRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: true,
        tools: if tools.is_empty() {
            None
        } else {
            Some(tools.to_vec())
        },
        options: None,
        keep_alive: Some(keep_alive),
        think: Some(think),
    }
}

pub fn build_assistant_message(result: &StreamResult) -> ChatMessage {
    let tool_calls = if result.tool_calls.is_empty() {
        None
    } else {
        Some(
            result
                .tool_calls
                .iter()
                .enumerate()
                .map(|(i, (name, args))| ToolCallOllama {
                    id: result.tool_call_ids.get(i).cloned(),
                    extra_content: result.tool_call_extra_content.get(i).cloned().flatten(),
                    function: ToolCallFunction {
                        name: name.clone(),
                        arguments: args.clone(),
                    },
                })
                .collect(),
        )
    };
    let reasoning = if result.thinking.is_empty() {
        None
    } else {
        Some(result.thinking.clone())
    };
    ChatMessage {
        role: "assistant".to_string(),
        content: result.content.clone(),
        tool_calls,
        reasoning_content: reasoning,
        ..Default::default()
    }
}

pub async fn decharge_gpu(model: &str) {
    let keep_alive = crate::services::config::read_config()
        .map(|c| c.advanced.keep_alive)
        .unwrap_or_else(|_| "5m".to_string());
    if keep_alive != "0" {
        return;
    }
    let client = reqwest::Client::new();
    let _ = client
        .post(format!("{}/api/chat", ollama_base_url()))
        .json(&serde_json::json!({
            "model": model,
            "messages": [],
            "keep_alive": "0"
        }))
        .send()
        .await;
}

pub async fn record_detected_tool_calls(
    session_id: &str,
    request_id: &str,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &Path,
) {
    for (name, args) in tool_calls {
        super::tool_executor_diagnostics::detected(session_id, request_id, name, args, working_dir)
            .await;
    }
}

pub async fn ensure_more_turns(turn: usize, model: &str) -> Result<(), String> {
    if turn == super::agent_loop_limits::MAX_TURNS - 1 {
        decharge_gpu(model).await;
        Err(super::agent_loop_errors::max_turns_message())
    } else {
        Ok(())
    }
}
