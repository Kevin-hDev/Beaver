use crate::services::agent_local::session_store;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentMessage;
use crate::services::compress::{context_capsules_disk, engine, prompt, token_estimate};
use std::path::Path;

pub use context_capsules_disk::CompressionMode;

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
    let mut session = session_store::get(session_id).await?;
    let context = context_capsules_disk::compression_context_message(
        runtime_messages,
        context_window,
        working_dir,
        mode,
    )
    .await;
    let (runtime_recent, session_recent) =
        super::state_recent::recent_messages(&session.messages, runtime_messages);

    replace_runtime_messages(
        runtime_messages,
        summary,
        suppress_follow_up,
        context.clone(),
        runtime_recent,
    );
    session.messages = build_session_messages(summary, suppress_follow_up, context, session_recent);
    session.accumulated_tokens =
        crate::services::token_counting::estimate_agent_messages_tokens(&session.messages);
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
    if let Some(context) = context {
        next.push(context);
    }
    next.extend(recent);
    *messages = next;
}

fn build_session_messages(
    summary: &str,
    suppress_follow_up: bool,
    context: Option<ChatMessage>,
    recent: Vec<AgentMessage>,
) -> Vec<AgentMessage> {
    let mut messages = vec![summary_agent_message(summary, suppress_follow_up)];
    if let Some(context) = context {
        messages.push(super::state_recent::chat_to_agent_message(&context));
    }
    messages.extend(recent);
    messages
}

fn summary_agent_message(summary: &str, suppress_follow_up: bool) -> AgentMessage {
    let content = prompt::format_summary_message(summary, suppress_follow_up);
    let chat = ChatMessage {
        role: "user".to_string(),
        content: content.clone(),
        ..Default::default()
    };
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: token_estimate::estimate_tokens(&[chat]) as u32,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}
