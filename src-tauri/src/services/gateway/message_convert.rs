use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
use crate::services::agent_local::types_session::{
    AgentMessage, AgentSession, ToolActivityRecord, ToolCallRequest,
};

const INVALID_SESSION_HISTORY: &str = "Historique de session invalide.";

pub fn build_chat_messages(session: &AgentSession) -> Result<Vec<ChatMessage>, String> {
    let mut messages = Vec::new();
    for message in session
        .messages
        .iter()
        .filter(|message| message.role != "system")
    {
        messages.extend(agent_to_chat_messages(message)?);
    }
    Ok(messages)
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
        thinking: m.display_thinking.clone(),
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

fn agent_to_chat_messages(m: &AgentMessage) -> Result<Vec<ChatMessage>, String> {
    if !matches!(m.role.as_str(), "system" | "user" | "assistant" | "tool") {
        return Err(INVALID_SESSION_HISTORY.to_string());
    }
    if let Some(segments) = &m.segments {
        let mut out = Vec::new();
        let mut id_counter = 0usize;
        for seg in segments {
            push_tool_turn(&mut out, &seg.tools, &seg.content, &mut id_counter);
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }
    if let Some(activities) = &m.tool_activities {
        let mut out = Vec::new();
        let mut id_counter = 0usize;
        push_tool_turn(&mut out, activities, &m.content, &mut id_counter);
        return Ok(out);
    }
    let message = match m.role.as_str() {
        "system" => ChatMessage::system(m.content.clone()),
        "user" => ChatMessage::user(m.content.clone()),
        "assistant" => ChatMessage::assistant(
            m.content.clone(),
            m.thinking.clone(),
            None,
            None,
            session_tool_calls_to_chat(m.tool_calls.as_ref()),
        ),
        "tool" => ChatMessage::tool(m.content.clone(), None, m.tool_name.clone()),
        _ => return Err(INVALID_SESSION_HISTORY.to_string()),
    };
    Ok(vec![message])
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
            None,
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
        out.push(ChatMessage::assistant(
            content.to_string(),
            None,
            None,
            None,
            None,
        ));
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
#[path = "message_convert_tests.rs"]
mod tests;
