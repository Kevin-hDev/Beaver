use super::types_session::{AgentSession, CloneMode};
use chrono::Utc;
use uuid::Uuid;

pub(super) fn build_clone(
    source: &AgentSession,
    message_id: &str,
    mode: CloneMode,
    prefix_end: usize,
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
    clone.messages = source.messages[..prefix_end].to_vec();
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
