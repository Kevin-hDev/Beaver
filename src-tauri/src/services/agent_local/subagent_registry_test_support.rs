use super::types_session::AgentSessionMeta;
use chrono::Utc;

pub fn meta(id: &str, status: &str) -> AgentSessionMeta {
    AgentSessionMeta {
        id: id.into(),
        name: "Geminitor".into(),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        archived_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        message_count: 0,
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        parent_session_id: Some("parent".into()),
        subagent_type: Some("explorer".into()),
        subagent_status: Some(status.into()),
        subagent_run_id: Some("saved-run".into()),
        subagent_description: Some("Analyse".into()),
        subagent_color_key: Some("geminitor".into()),
        subagent_summary: None,
        subagent_last_activity: None,
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_root_session_id: None,
        git_branch: None,
    }
}
