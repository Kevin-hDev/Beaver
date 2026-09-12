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
    pub prepared_count: crate::services::agent_local::context_usage_record::ContextTokenCount,
    pub system_head_count: crate::services::agent_local::context_usage_record::ContextTokenCount,
    pub transient_overhead_tokens: u32,
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
        let empty_count =
            super::prepared_request::count(&session.provider, &session.model, &[], &[]);
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
            prepared_count: empty_count.clone(),
            system_head_count: empty_count,
            transient_overhead_tokens: 0,
            provider_id: session.provider.clone(),
            source_session: session.clone(),
        })
    }

    #[cfg(test)]
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
        self.prepared_count =
            crate::services::agent_local::context_usage_record::ContextTokenCount {
                tokens: Some(before_tokens),
                capacity_tokens: Some(before_tokens),
                source: Some(
                    crate::services::agent_local::context_usage_record::ContextCountSource::Heuristic,
                ),
                coverage:
                    crate::services::agent_local::context_usage_record::ContextCountCoverage::Complete,
            };
        self.system_head_count = super::prepared_request::system_head(
            &self.provider_id,
            &self.source_session.model,
            &self.canonical_messages,
            &self.provider_tools,
        );
        Ok(self)
    }

    pub fn with_prepared_context(
        mut self,
        runtime_messages: &[ChatMessage],
        provider_tools: Vec<serde_json::Value>,
        prepared_count: crate::services::agent_local::context_usage_record::ContextTokenCount,
    ) -> Result<Self, super::checkpoint_transaction::CompressionError> {
        if provider_tools.len() > 256 {
            return Err(super::checkpoint_transaction::CompressionError::SnapshotInvalid);
        }
        let baseline = super::prepared_request::count(
            &self.provider_id,
            &self.source_session.model,
            runtime_messages,
            &provider_tools,
        )
        .capacity_tokens
        .ok_or(super::checkpoint_transaction::CompressionError::CapacityUnverified)?;
        self.canonical_messages = runtime_messages
            .iter()
            .filter(|message| matches!(message.role.as_str(), "system" | "developer"))
            .cloned()
            .collect();
        if self.canonical_messages.len() > 64 {
            return Err(super::checkpoint_transaction::CompressionError::SnapshotInvalid);
        }
        self.provider_tools = provider_tools;
        self.transient_overhead_tokens = prepared_count
            .capacity_tokens
            .ok_or(super::checkpoint_transaction::CompressionError::CapacityUnverified)?
            .saturating_sub(baseline);
        self.prepared_count = prepared_count;
        self.system_head_count = super::prepared_request::system_head(
            &self.provider_id,
            &self.source_session.model,
            &self.canonical_messages,
            &self.provider_tools,
        );
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

    pub fn before_tokens(&self) -> u32 {
        self.prepared_count.capacity_tokens.unwrap_or_default()
    }

    pub fn system_head_tokens(&self) -> u32 {
        self.system_head_count.capacity_tokens.unwrap_or_default()
    }

    pub fn capacity_verified(&self) -> bool {
        self.prepared_count.capacity_tokens.is_some()
            && self.system_head_count.capacity_tokens.is_some()
    }
}
