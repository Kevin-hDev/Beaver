#![allow(
    dead_code,
    reason = "the compression orchestrator consumes this staged reconstruction in Task 10"
)]

use serde::Serialize;

use super::session_capabilities::SessionCompressionCapabilities;
use super::snapshot::CompressionSnapshot;
use crate::services::agent_local::types_message::AgentMessage;
use crate::services::agent_local::types_session::AgentSession;

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalCompressionContext {
    pub profile_id: String,
    pub profile_revision: u64,
    pub context_window: u64,
    pub trigger: super::profile_types::CompressionTrigger,
    pub tool_names: Vec<String>,
    pub capabilities: SessionCompressionCapabilities,
    pub messages: Vec<AgentMessage>,
}

pub async fn rebuild_canonical_context(
    session: &AgentSession,
    request: &CompressionSnapshot,
) -> Result<CanonicalCompressionContext, String> {
    if session.id != request.session_id {
        return Err("compression_snapshot_invalid".to_string());
    }
    let messages = request
        .source_messages
        .iter()
        .filter(|message| message.role != "system")
        .cloned()
        .collect::<Vec<_>>();
    crate::services::agent_local::conversation_history_validation::validate(&messages)
        .map_err(|_| "compression_snapshot_invalid".to_string())?;
    Ok(CanonicalCompressionContext {
        profile_id: request.profile.profile.id.clone(),
        profile_revision: request.profile.profile_revision,
        context_window: request.context_window,
        trigger: request.trigger,
        tool_names: request.capabilities.tool_names.iter().cloned().collect(),
        capabilities: request.capabilities.clone(),
        messages,
    })
}
