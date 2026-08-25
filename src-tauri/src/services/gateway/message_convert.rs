use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
use crate::services::agent_local::types_session::{
    AgentMessage, AgentSession, ToolActivityRecord, ToolCallRequest,
};

pub fn build_chat_messages(session: &AgentSession) -> Vec<ChatMessage> {
    session
        .messages
        .iter()
        .filter(|m| m.role != "system")
        .flat_map(agent_to_chat_messages)
        .collect()
}

pub fn new_user_message(content: &str) -> ChatMessage {
    ChatMessage::user(content.to_string())
}

pub fn chat_to_agent_message(m: &ChatMessage) -> Option<AgentMessage> {
    if m.role == "system" {
        return None;
    }
    Some(AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: m.role.clone(),
        content: m.content.clone(),
        thinking: None,
        tool_calls: m
            .tool_calls
            .as_ref()
            .map(|calls| chat_tool_calls_to_session(calls)),
        tool_name: m.tool_name.clone(),
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    })
}

fn agent_to_chat_messages(m: &AgentMessage) -> Vec<ChatMessage> {
    if let Some(segments) = &m.segments {
        let mut out = Vec::new();
        let mut id_counter = 0usize;
        for seg in segments {
            push_tool_turn(&mut out, &seg.tools, &seg.content, &mut id_counter);
        }
        if !out.is_empty() {
            return out;
        }
    }
    if let Some(activities) = &m.tool_activities {
        let mut out = Vec::new();
        let mut id_counter = 0usize;
        push_tool_turn(&mut out, activities, &m.content, &mut id_counter);
        return out;
    }
    let message = match m.role.as_str() {
        "system" => ChatMessage::system(m.content.clone()),
        "user" => ChatMessage::user(m.content.clone()),
        "assistant" => ChatMessage::assistant(
            m.content.clone(),
            m.thinking.clone(),
            session_tool_calls_to_chat(m.tool_calls.as_ref()),
        ),
        "tool" => ChatMessage::tool(m.content.clone(), None, m.tool_name.clone()),
        _ => return Vec::new(),
    };
    vec![message]
}

fn push_tool_turn(
    out: &mut Vec<ChatMessage>,
    tools: &[ToolActivityRecord],
    content: &str,
    id_counter: &mut usize,
) {
    let tool_calls: Vec<_> = tools
        .iter()
        .map(|tool| {
            let id = format!("restored-{}", *id_counter);
            *id_counter += 1;
            ToolCallOllama {
                id: Some(id),
                extra_content: None,
                function: ToolCallFunction {
                    name: tool.name.clone(),
                    arguments: tool.args.clone().unwrap_or_default(),
                },
            }
        })
        .collect();
    if !tool_calls.is_empty() {
        out.push(ChatMessage::assistant(
            String::new(),
            None,
            Some(tool_calls.clone()),
        ));
        for (tool, call) in tools.iter().zip(tool_calls.iter()) {
            out.push(ChatMessage::tool(
                tool.result.clone().unwrap_or_default(),
                call.id.clone(),
                Some(tool.name.clone()),
            ));
        }
    }
    if !content.is_empty() {
        out.push(ChatMessage::assistant(content.to_string(), None, None));
    }
}

fn session_tool_calls_to_chat(calls: Option<&Vec<ToolCallRequest>>) -> Option<Vec<ToolCallOllama>> {
    calls.map(|items| {
        items
            .iter()
            .map(|call| ToolCallOllama {
                id: None,
                extra_content: call.extra_content.clone(),
                function: ToolCallFunction {
                    name: call.function.name.clone(),
                    arguments: call.function.arguments.clone(),
                },
            })
            .collect()
    })
}

fn chat_tool_calls_to_session(calls: &[ToolCallOllama]) -> Vec<ToolCallRequest> {
    calls
        .iter()
        .map(|call| ToolCallRequest {
            extra_content: call.extra_content.clone(),
            function: crate::services::agent_local::types_session::ToolCallRequestFunction {
                name: call.function.name.clone(),
                arguments: call.function.arguments.clone(),
            },
        })
        .collect()
}

pub fn new_user_agent_message(content: &str) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: content.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_history_keeps_tool_calls_and_results() {
        let call = ToolCallOllama {
            id: Some("call-1".into()),
            extra_content: None,
            function: ToolCallFunction {
                name: "read_file".into(),
                arguments: serde_json::json!({"path":"README.md"}),
            },
        };
        let assistant = ChatMessage::assistant(String::new(), None, Some(vec![call]));
        let tool = ChatMessage::tool(
            "Beaver".into(),
            Some("call-1".into()),
            Some("read_file".into()),
        );

        let saved_call = chat_to_agent_message(&assistant).unwrap();
        let saved_result = chat_to_agent_message(&tool).unwrap();

        assert_eq!(saved_call.tool_calls.unwrap()[0].function.name, "read_file");
        assert_eq!(saved_result.role, "tool");
        assert_eq!(saved_result.tool_name.as_deref(), Some("read_file"));
        assert_eq!(saved_result.content, "Beaver");
    }
}
