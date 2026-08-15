use crate::services::agent_local::types_ollama::{
    ChatMessage, ChatRequest, OllamaThink, StreamResult, ToolCallFunction, ToolCallOllama,
};

pub async fn prepare_subagents(
    session_id: &str,
    parent_message_inbox: Option<
        std::sync::Arc<super::parent_message_inbox::ParentMessageInbox>,
    >,
) -> super::subagent_orchestration::ParentSubagentOrchestrator {
    super::tool_result_budget::cleanup_old_results();
    super::subagent_orchestration::ParentSubagentOrchestrator::with_parent_inbox(
        session_id,
        parent_message_inbox,
    )
    .await
}

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

pub fn build_for_plan(result: &StreamResult, plan_active: bool) -> ChatMessage {
    let mut message = build_assistant_message(result);
    if plan_active && !result.tool_calls.is_empty() {
        message.content.clear();
    }
    message
}

pub async fn decharge_gpu(model: &str) {
    let keep_alive = crate::services::config::read_config()
        .map(|c| c.advanced.keep_alive)
        .unwrap_or_else(|_| "5m".to_string());
    if keep_alive != "0" {
        return;
    }
    let Ok(ollama) = super::ollama_client::OllamaClient::from_global() else {
        return;
    };
    let Ok(base_url) = ollama.base_url().await else {
        return;
    };
    let client = reqwest::Client::new();
    let _ = client
        .post(format!("{base_url}/api/chat"))
        .json(&serde_json::json!({
            "model": model,
            "messages": [],
            "keep_alive": "0"
        }))
        .send()
        .await;
}

pub async fn ensure_more_turns(turn: usize, model: &str) -> Result<(), String> {
    if turn == super::agent_loop_limits::MAX_TURNS - 1 {
        decharge_gpu(model).await;
        Err(super::agent_loop_errors::max_turns_message())
    } else {
        Ok(())
    }
}
