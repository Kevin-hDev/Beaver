use super::types_session::AgentSession;
use crate::services::reasoning_profile::EffectiveReasoningProfile;

#[derive(Debug, Clone)]
pub(crate) struct SessionReasoningUpdate {
    provider: String,
    model: String,
    expected_mode: Option<String>,
    expected_enabled: bool,
    effective_mode: Option<String>,
    effective_enabled: bool,
}

impl SessionReasoningUpdate {
    pub(crate) fn new(session: &AgentSession, profile: &EffectiveReasoningProfile) -> Self {
        Self {
            provider: session.provider.clone(),
            model: session.model.clone(),
            expected_mode: session.reasoning_mode.clone(),
            expected_enabled: session.thinking_enabled,
            effective_mode: profile.mode_name.clone(),
            effective_enabled: profile.active,
        }
    }

    pub(crate) fn apply(&self, session: &mut AgentSession) -> Result<bool, ()> {
        if session.provider != self.provider
            || session.model != self.model
            || session.reasoning_mode != self.expected_mode
            || session.thinking_enabled != self.expected_enabled
        {
            return Err(());
        }
        let changed = session.reasoning_mode != self.effective_mode
            || session.thinking_enabled != self.effective_enabled;
        session.reasoning_mode.clone_from(&self.effective_mode);
        session.thinking_enabled = self.effective_enabled;
        Ok(changed)
    }
}
