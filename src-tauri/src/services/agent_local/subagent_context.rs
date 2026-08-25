use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::agent_local::types_session::{AgentMessage, ToolCallRequest};

pub async fn build_messages(
    child_session_id: &str,
    system_prompt: String,
    fallback_prompt: &str,
    prior_messages: Option<Vec<ChatMessage>>,
) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::system(system_prompt)];

    if let Some(prior) = prior_messages {
        messages.extend(prior.into_iter().filter(|message| message.role != "system"));
    } else if let Ok(child) = super::session_store::get(child_session_id).await {
        messages.extend(child.messages.into_iter().filter_map(saved_to_chat));
    }

    if messages.len() == 1 {
        messages.push(ChatMessage::user(fallback_prompt.to_string()));
    }

    messages
}

fn saved_to_chat(message: AgentMessage) -> Option<ChatMessage> {
    if !matches!(message.role.as_str(), "user" | "assistant" | "tool") {
        return None;
    }
    let tool_calls = message.tool_calls.map(convert_tool_calls);
    if message.role != "tool"
        && message.content.trim().is_empty()
        && tool_calls.as_ref().is_none_or(Vec::is_empty)
    {
        return None;
    }
    match message.role.as_str() {
        "user" => Some(ChatMessage::user(message.content)),
        "assistant" => Some(ChatMessage::assistant(
            message.content,
            message.thinking,
            message.continuation,
            None,
            tool_calls,
        )),
        "tool" => Some(ChatMessage::tool(
            message.content,
            message.tool_call_id,
            message.tool_name,
        )),
        _ => None,
    }
}

fn convert_tool_calls(calls: Vec<ToolCallRequest>) -> Vec<ToolCallOllama> {
    calls
        .into_iter()
        .map(|call| ToolCallOllama {
            id: Some(call.id),
            extra_content: call.extra_content,
            function: ToolCallFunction {
                name: call.function.name,
                arguments: call.function.arguments,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(role: &str, content: &str) -> AgentMessage {
        AgentMessage {
            id: "m1".into(),
            turn_id: "turn-1".into(),
            role: role.into(),
            content: content.into(),
            thinking: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            continuation: None,
            tool_activities: None,
            segments: None,
            files: vec![],
            timestamp: chrono::Utc::now(),
            tokens: 0,
            work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
        }
    }

    #[test]
    fn saved_to_chat_keeps_supported_roles() {
        assert!(saved_to_chat(saved("user", "Suite")).is_some());
        assert!(saved_to_chat(saved("assistant", "Ok")).is_some());
        assert!(saved_to_chat(saved("system", "Ignore")).is_none());
    }

    #[tokio::test]
    async fn in_memory_history_keeps_provider_tool_ids_under_one_fresh_system_message() {
        let prior = vec![
            ChatMessage::system("ancien système".into()),
            ChatMessage::assistant("appel".into(), None, None, None, Some(vec![ToolCallOllama {
                    id: Some("call-exact".into()),
                    extra_content: None,
                    function: ToolCallFunction {
                        name: "read_file".into(),
                        arguments: serde_json::json!({"path": "README.md"}),
                    },
                }])),
            ChatMessage::tool("résultat".into(), Some("call-exact".into()), Some("read_file".into())),
        ];

        let messages = build_messages("missing", "nouveau système".into(), "fallback", Some(prior))
            .await;

        assert_eq!(messages.iter().filter(|message| message.role == "system").count(), 1);
        assert_eq!(messages[0].content, "nouveau système");
        assert_eq!(messages[1].tool_calls.as_ref().unwrap()[0].id.as_deref(), Some("call-exact"));
        assert_eq!(messages[2].tool_call_id.as_deref(), Some("call-exact"));
    }
}
