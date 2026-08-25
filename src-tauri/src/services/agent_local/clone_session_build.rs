use super::clone_summary;
use super::types_session::{AgentMessage, AgentSession, CloneMode};
use chrono::Utc;
use uuid::Uuid;

pub(super) fn build_clone(
    source: &AgentSession,
    message_id: &str,
    mode: CloneMode,
    message_index: usize,
    root_session_id: &str,
) -> AgentSession {
    let now = Utc::now();
    let mut clone = source.clone();
    clone.id = Uuid::new_v4().to_string();
    clone.name = format!("Clone - {}", source.name);
    clone.created_at = now;
    clone.updated_at = Some(now);
    clone.archived_at = None;
    // Un clone est une nouvelle session : il ne reprend pas une préférence de coût/vitesse.
    clone.fast_mode_enabled = false;
    clone.messages = source.messages[..=message_index].to_vec();
    super::session_store_messages::recompute_accumulated_tokens(&mut clone);
    clone.stream_failures.clear();
    clone.diagnostic_runs.clear();
    clone.clone_parent_session_id = Some(source.id.clone());
    clone.clone_parent_message_id = Some(message_id.to_string());
    clone.clone_mode = Some(mode);
    clone.clone_summary = None;
    clone.clone_read_files.clear();
    clone.clone_modified_files.clear();
    clone.clone_root_session_id = Some(root_session_id.to_string());
    clone.git_branch = None;
    clone.parent_session_id = None;
    clone.subagent_type = None;
    clone.subagent_worktree = None;
    clone.subagent_prompt = None;
    clone.subagent_status = None;
    clone.subagent_run_id = None;
    clone.subagent_description = None;
    clone.subagent_color_key = None;
    clone.subagent_summary = None;
    clone.subagent_queued_prompts.clear();
    clone.subagent_hidden_reports.clear();
    clone
}

pub(super) fn hidden_context_message(summary: &str) -> AgentMessage {
    AgentMessage {
        id: Uuid::new_v4().to_string(),
        turn_id: AgentMessage::new_turn_id(),
        role: "user".to_string(),
        content: clone_summary::hidden_context_content(summary),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}
