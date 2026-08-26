use crate::services::agent_local::session_store;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentMessage;
use crate::services::compress::{context_capsules_disk, engine, prompt, token_estimate};
use std::path::Path;

pub use context_capsules_disk::CompressionMode;

const RECENT_COMPLETE_TURNS: usize = 2;

pub fn context_used_for_compression(
    last_context_tokens: Option<u32>,
    estimated_tokens: usize,
) -> usize {
    last_context_tokens
        .map(|tokens| std::cmp::max(tokens as usize, estimated_tokens))
        .unwrap_or(estimated_tokens)
}

pub fn is_safe_to_compress(messages: &[ChatMessage]) -> bool {
    super::state_recent::tool_chain_is_closed(messages)
}

pub async fn apply_and_save(
    session_id: &str,
    runtime_messages: &mut Vec<ChatMessage>,
    summary: &str,
    context_window: u64,
    suppress_follow_up: bool,
    working_dir: &Path,
    mode: CompressionMode,
) -> Result<u32, String> {
    let lock = session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = session_store::get(session_id).await?;
    let context = context_capsules_disk::compression_context_message(
        runtime_messages,
        context_window,
        working_dir,
        mode,
    )
    .await;
    let runtime_recent = current_runtime_turn(runtime_messages);

    replace_runtime_messages(
        runtime_messages,
        summary,
        suppress_follow_up,
        context.clone(),
        runtime_recent,
    );
    session
        .messages
        .retain(|message| !(message.role == "user" && message.content.trim() == "/compress"));
    crate::services::agent_local::conversation_compaction::compact_complete_turns(
        &mut session.messages,
        RECENT_COMPLETE_TURNS,
    )
    .map_err(|_| "Compression impossible".to_string())?;
    let mut compacted = build_summary_turn(summary, suppress_follow_up, context);
    compacted.append(&mut session.messages);
    session.messages = compacted;
    crate::services::agent_local::session_store_messages::recompute_accumulated_tokens(
        &mut session,
    );
    session_store::save(&session).await?;

    Ok(token_estimate::estimate_tokens(runtime_messages) as u32)
}

pub fn request_start_index(messages: &[ChatMessage]) -> usize {
    messages
        .iter()
        .rposition(|message| {
            message.role == "user" && super::state_recent::include_chat_message(message)
        })
        .unwrap_or(0)
}

fn replace_runtime_messages(
    messages: &mut Vec<ChatMessage>,
    summary: &str,
    suppress_follow_up: bool,
    context: Option<ChatMessage>,
    recent: Vec<ChatMessage>,
) {
    let system_messages: Vec<_> = messages
        .iter()
        .filter(|message| message.role == "system")
        .cloned()
        .collect();
    let mut next = system_messages;
    next.extend(engine::build_post_compression_messages(
        summary,
        suppress_follow_up,
    ));
    next.push(ChatMessage::assistant(
        engine::BOUNDARY_CONTENT.to_string(),
        None,
        None,
        None,
        None,
    ));
    if let Some(context) = context {
        if let Some(summary_message) = next.iter_mut().rev().find(|message| message.role == "user")
        {
            summary_message.content.push_str("\n\n");
            summary_message.content.push_str(&context.content);
        }
    }
    let boundary_index = next.len();
    next.extend(recent);
    if let Some(message) = next.get_mut(boundary_index) {
        message.continuity_barrier_before = true;
    }
    *messages = next;
}

fn build_summary_turn(
    summary: &str,
    suppress_follow_up: bool,
    context: Option<ChatMessage>,
) -> Vec<AgentMessage> {
    let turn_id = AgentMessage::new_turn_id();
    let mut user = summary_agent_message(summary, suppress_follow_up, turn_id.clone());
    if let Some(context) = context {
        user.content.push_str("\n\n");
        user.content.push_str(&context.content);
    }
    let assistant = AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id,
        role: "assistant".to_string(),
        content: engine::BOUNDARY_CONTENT.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    };
    vec![user, assistant]
}

fn summary_agent_message(summary: &str, suppress_follow_up: bool, turn_id: String) -> AgentMessage {
    let content = prompt::format_summary_message(summary, suppress_follow_up);
    let chat = ChatMessage::user(content.clone());
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id,
        role: "user".to_string(),
        content,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: token_estimate::estimate_tokens(&[chat]) as u32,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}

fn current_runtime_turn(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    let start = request_start_index(messages);
    messages[start..]
        .iter()
        .filter(|message| super::state_recent::include_chat_message(message))
        .cloned()
        .collect()
}
