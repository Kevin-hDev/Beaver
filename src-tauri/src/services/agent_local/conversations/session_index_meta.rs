use super::session_security;
use super::types_session::{AgentSession, AgentSessionMeta};

pub(crate) fn from_session(session: &AgentSession) -> AgentSessionMeta {
    AgentSessionMeta {
        id: session.id.clone(),
        name: super::sensitive_data::redact_high_confidence_text(&session.name),
        created_at: session.created_at,
        updated_at: session.updated_at,
        archived_at: session.archived_at,
        pinned_at: session.pinned_at,
        model: session.model.clone(),
        provider: session.provider.clone(),
        thinking_enabled: session.thinking_enabled,
        fast_mode_enabled: session.fast_mode_enabled,
        reasoning_mode: session.reasoning_mode.clone(),
        message_count: session.messages.len(),
        is_heartbeat: session.is_heartbeat,
        is_gateway: session.is_gateway,
        has_active_context_request: session.context_usage.active_request_id.is_some(),
        gateway_channel_key: session_security::redacted_optional(&session.gateway_channel_key),
        project_id: session.project_id.clone(),
        parent_session_id: session.parent_session_id.clone(),
        subagent_type: session.subagent_type.clone(),
        subagent_status: session.subagent_status.clone(),
        subagent_run_id: session.subagent_run_id.clone(),
        subagent_description: session_security::redacted_optional(&session.subagent_description),
        subagent_color_key: session.subagent_color_key.clone(),
        subagent_summary: session_security::redacted_optional(&session.subagent_summary),
        subagent_last_activity: session_security::redacted_activity(
            &session.subagent_last_activity,
        ),
        clone_parent_session_id: session.clone_parent_session_id.clone(),
        clone_parent_message_id: session.clone_parent_message_id.clone(),
        clone_mode: session.clone_mode.clone(),
        clone_root_session_id: session.clone_root_session_id.clone(),
        git_branch: session.git_branch.clone(),
    }
}
