#![allow(
    dead_code,
    reason = "the compression orchestrator consumes this staged snapshot in Task 10"
)]

use super::profile_resolve::ResolvedCompressionProfile;
use super::profile_types::CompressionTrigger;
use super::session_capabilities::SessionCompressionCapabilities;
use crate::services::agent_local::types_message::AgentMessage;
use crate::services::agent_local::types_session::AgentSession;

#[derive(Debug, Clone)]
pub struct CompressionSnapshot {
    pub session_id: String,
    pub source_messages: Vec<AgentMessage>,
    pub profile: ResolvedCompressionProfile,
    pub context_window: u64,
    pub capabilities: SessionCompressionCapabilities,
    pub trigger: CompressionTrigger,
}

impl CompressionSnapshot {
    pub fn capture(
        session: &AgentSession,
        profile: ResolvedCompressionProfile,
        context_window: u64,
        capabilities: SessionCompressionCapabilities,
        trigger: CompressionTrigger,
    ) -> Result<Self, String> {
        crate::services::agent_local::session_store::validate_session_id(&session.id)?;
        if session.messages.len()
            > crate::services::agent_local::session_limits::MAX_MESSAGES_PER_SESSION
        {
            return Err("compression_snapshot_invalid".to_string());
        }
        Ok(Self {
            session_id: session.id.clone(),
            source_messages: session.messages.clone(),
            profile,
            context_window,
            capabilities,
            trigger,
        })
    }
}
