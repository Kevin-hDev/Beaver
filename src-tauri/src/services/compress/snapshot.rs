use super::profile_resolve::ResolvedCompressionProfile;
use super::profile_types::CompressionTrigger;
use super::session_capabilities::SessionCompressionCapabilities;
use crate::services::agent_local::types_message::AgentMessage;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentSession;

#[derive(Debug, Clone)]
pub struct CompressionSnapshot {
    pub session_id: String,
    pub source_messages: Vec<AgentMessage>,
    pub profile: ResolvedCompressionProfile,
    pub context_window: u64,
    pub capabilities: SessionCompressionCapabilities,
    pub trigger: CompressionTrigger,
    pub canonical_messages: Vec<ChatMessage>,
    pub provider_tools: Vec<serde_json::Value>,
    pub checkpoint_images: Vec<super::checkpoint_attachments::CheckpointImage>,
    pub before_tokens: u32,
    pub system_head_tokens: u32,
    pub provider_id: String,
    pub(crate) source_session: AgentSession,
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
        let before_tokens =
            crate::services::token_counting::estimate_agent_messages_tokens(&session.messages);
        Ok(Self {
            session_id: session.id.clone(),
            source_messages: session.messages.clone(),
            profile,
            context_window,
            capabilities,
            trigger,
            canonical_messages: Vec::new(),
            provider_tools: Vec::new(),
            checkpoint_images: Vec::new(),
            before_tokens,
            system_head_tokens: 0,
            provider_id: session.provider.clone(),
            source_session: session.clone(),
        })
    }

    pub fn with_runtime_context(
        mut self,
        canonical_messages: Vec<ChatMessage>,
        provider_tools: Vec<serde_json::Value>,
        before_tokens: u32,
    ) -> Result<Self, String> {
        if canonical_messages.len() > 64 || provider_tools.len() > 256 {
            return Err("compression_snapshot_invalid".to_string());
        }
        self.canonical_messages = canonical_messages;
        self.provider_tools = provider_tools;
        self.before_tokens = before_tokens;
        self.system_head_tokens =
            super::token_estimate::estimate_textual_request_tokens_for_provider(
                &self.provider_id,
                &self.canonical_messages,
                &self.provider_tools,
            )
            .min(u32::MAX as usize) as u32;
        Ok(self)
    }

    pub fn with_checkpoint_images(
        mut self,
        images: Vec<super::checkpoint_attachments::CheckpointImage>,
    ) -> Result<Self, String> {
        if images.len() > super::checkpoint_attachments::MAX_IMAGE_CANDIDATES {
            return Err("compression_snapshot_invalid".to_string());
        }
        self.checkpoint_images = images;
        Ok(self)
    }
}
