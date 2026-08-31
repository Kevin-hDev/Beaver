use super::types_message::AgentMessage;
use super::types_session::AgentSession;

const INTERRUPTED_TOOL_RESULT: &str = r#"{"status":"cancelled","error":"tool_interrupted"}"#;

/// Une forme v1 irrégulière reste lisible : on conserve le plus long suffixe
/// de tours valide, au lieu de livrer une session qui ne peut plus continuer.
pub(super) fn repair(session: &mut AgentSession) {
    if valid(session) {
        return;
    }
    merge_terminal_assistants_before_tool_calls(session);
    close_interrupted_tool_turn(session);
    close_completed_tool_turns(session);
    if valid(session) {
        return;
    }
    let start = session
        .messages
        .iter()
        .enumerate()
        .filter(|(_, message)| message.role == "user")
        .map(|(index, _)| index)
        .find(|index| {
            super::conversation_history_validation::validate(&session.messages[*index..]).is_ok()
        });
    if let Some(start) = start {
        session.messages.drain(..start);
    }
}

fn valid(session: &AgentSession) -> bool {
    super::conversation_history_validation::validate(&session.messages).is_ok()
}

/// Les sessions v1 pouvaient persister le texte assistant, puis l'appel outil
/// dans un second message assistant du même tour. Le contrat v2 les regroupe.
fn merge_terminal_assistants_before_tool_calls(session: &mut AgentSession) {
    let mut index = 1usize;
    while index < session.messages.len() {
        let should_merge = session.messages[index - 1].role == "assistant"
            && session.messages[index - 1].tool_calls.is_none()
            && session.messages[index].role == "assistant"
            && session.messages[index]
                .tool_calls
                .as_ref()
                .is_some_and(|calls| !calls.is_empty())
            && session.messages[index - 1].turn_id == session.messages[index].turn_id;
        if !should_merge {
            index += 1;
            continue;
        }
        let terminal = session.messages.remove(index - 1);
        let tool_message = &mut session.messages[index - 1];
        if !terminal.content.is_empty() {
            let mut content = terminal.content;
            if !tool_message.content.is_empty() {
                content.push_str("\n\n");
                content.push_str(&tool_message.content);
            }
            tool_message.content = content;
        }
        if tool_message.thinking.is_none() {
            tool_message.thinking = terminal.thinking;
        }
    }
}

fn close_interrupted_tool_turn(session: &mut AgentSession) {
    let Some(call_index) = session.messages.iter().rposition(|message| {
        message.role == "assistant"
            && message
                .tool_calls
                .as_ref()
                .is_some_and(|calls| !calls.is_empty())
    }) else {
        return;
    };
    if session.messages[call_index + 1..]
        .iter()
        .any(|message| message.role != "tool")
    {
        return;
    }
    let call_message = &session.messages[call_index];
    let turn_id = call_message.turn_id.clone();
    let timestamp = call_message.timestamp;
    let calls = call_message.tool_calls.clone().unwrap_or_default();
    let completed = session.messages[call_index + 1..]
        .iter()
        .filter_map(|message| message.tool_call_id.as_deref())
        .collect::<std::collections::HashSet<_>>();
    let missing = calls
        .into_iter()
        .filter(|call| !completed.contains(call.id.as_str()))
        .collect::<Vec<_>>();
    if missing.is_empty()
        || session
            .messages
            .len()
            .saturating_add(missing.len())
            .saturating_add(1)
            > super::session_limits::MAX_MESSAGES_PER_SESSION
    {
        return;
    }
    for call in missing {
        session.messages.push(interrupted_tool_result(
            &turn_id,
            timestamp,
            call.id,
            call.function.name,
        ));
    }
}

fn interrupted_tool_result(
    turn_id: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
    tool_call_id: String,
    tool_name: String,
) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: turn_id.to_string(),
        role: "tool".into(),
        content: INTERRUPTED_TOOL_RESULT.into(),
        message_kind: None,
        thinking: None,
        tool_calls: None,
        tool_name: Some(tool_name),
        tool_call_id: Some(tool_call_id),
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp,
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}

fn close_completed_tool_turns(session: &mut AgentSession) {
    let mut index = 1usize;
    while index < session.messages.len()
        && session.messages.len() < super::session_limits::MAX_MESSAGES_PER_SESSION
    {
        if session.messages[index].role == "user" && session.messages[index - 1].role == "tool" {
            let boundary = terminal_assistant(&session.messages[index - 1]);
            session.messages.insert(index, boundary);
            index += 1;
        }
        index += 1;
    }
    if session.messages.len() < super::session_limits::MAX_MESSAGES_PER_SESSION
        && session
            .messages
            .last()
            .is_some_and(|message| message.role == "tool")
    {
        let boundary = terminal_assistant(session.messages.last().expect("checked last message"));
        session.messages.push(boundary);
    }
}

fn terminal_assistant(tool: &AgentMessage) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: tool.turn_id.clone(),
        role: "assistant".into(),
        content: String::new(),
        message_kind: None,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: tool.timestamp,
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}
