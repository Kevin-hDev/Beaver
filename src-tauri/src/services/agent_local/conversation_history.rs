#![allow(dead_code, reason = "provider adapters adopt the canonical history in Tasks 9 and 18-23")]

use std::fmt;

use crate::services::reasoning_continuity::contract::{ContinuationTarget, ReplayTarget};
use crate::services::reasoning_continuity::envelope::ReasoningEnvelope;

use super::conversation_attachments::ResolvedImage;
use super::types_message::{FileAttachment, ToolCallRequest};

pub const PUBLIC_ERROR_CODE: &str = "conversation_admission_failed";
pub const SKILL_INSTRUCTION_PREFIX: &str =
    "The user has loaded the following skill. Follow its instructions exactly:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversationHistoryError;

impl fmt::Display for ConversationHistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(PUBLIC_ERROR_CODE)
    }
}

impl std::error::Error for ConversationHistoryError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderRole {
    User,
    Assistant,
    Tool,
}

#[derive(Debug)]
#[allow(dead_code, reason = "provider adapters adopt every field in Tasks 9 and 18-23")]
pub struct ProviderMessage {
    /// Absent seulement pour une instruction de skill éphémère du tour courant.
    pub message_id: Option<String>,
    pub turn_id: String,
    pub role: ProviderRole,
    pub content: String,
    pub images: Vec<ResolvedImage>,
    pub files: Vec<FileAttachment>,
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub display_thinking: Option<String>,
    pub continuation: Option<ReasoningEnvelope>,
    pub legacy_tool_loop_reasoning: Option<String>,
    pub skill_id: Option<String>,
    pub skill_name: Option<String>,
    pub continuity_barrier_before: bool,
}

#[derive(Debug)]
pub struct ConversationHistory {
    pub messages: Vec<ProviderMessage>,
    pub compatible_suffix_start: usize,
}

impl ConversationHistory {
    pub(crate) fn overlay_current_input(
        &mut self,
        user_message_id: &str,
        input: super::conversation_input::ResolvedTurnInput,
    ) -> Result<(), ConversationHistoryError> {
        let index = self
            .messages
            .iter()
            .rposition(|message| message.message_id.as_deref() == Some(user_message_id))
            .ok_or(ConversationHistoryError)?;
        if index + 1 != self.messages.len() || self.messages[index].role != ProviderRole::User {
            return Err(ConversationHistoryError);
        }
        self.overlay_at(index, input)
    }

    pub(super) fn overlay_resolved_input(
        &mut self,
        user_message_id: &str,
        input: super::conversation_input::ResolvedTurnInput,
    ) -> Result<(), ConversationHistoryError> {
        let index = self
            .messages
            .iter()
            .position(|message| message.message_id.as_deref() == Some(user_message_id))
            .ok_or(ConversationHistoryError)?;
        if self.messages[index].role != ProviderRole::User {
            return Err(ConversationHistoryError);
        }
        self.overlay_at(index, input)
    }

    fn overlay_at(
        &mut self,
        index: usize,
        input: super::conversation_input::ResolvedTurnInput,
    ) -> Result<(), ConversationHistoryError> {
        let super::conversation_input::ResolvedTurnInput {
            user_content: _,
            provider_content,
            files: _,
            images,
            skills,
        } = input;
        let turn_id = self.messages[index].turn_id.clone();
        self.messages[index].content = provider_content;
        self.messages[index].images = images;
        let inherited_barrier = self.messages[index].continuity_barrier_before;
        self.messages[index].continuity_barrier_before = inherited_barrier && skills.is_empty();

        let skill_messages = skills.into_iter().map(|skill| ProviderMessage {
            message_id: None,
            turn_id: turn_id.clone(),
            role: ProviderRole::User,
            content: format!("{SKILL_INSTRUCTION_PREFIX}\n\n{}", skill.content),
            images: Vec::new(),
            files: Vec::new(),
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            display_thinking: None,
            continuation: None,
            legacy_tool_loop_reasoning: None,
            skill_id: Some(skill.id),
            skill_name: Some(skill.name),
            continuity_barrier_before: false,
        });
        let count = skill_messages.len();
        self.messages.splice(index..index, skill_messages);
        if inherited_barrier && count > 0 {
            self.messages[index].continuity_barrier_before = true;
        }
        if self.compatible_suffix_start > index {
            self.compatible_suffix_start = self.compatible_suffix_start.saturating_add(count);
        }
        Ok(())
    }
}

pub async fn load_for_target(
    session_id: &str,
    target: &ReplayTarget,
) -> Result<ConversationHistory, ConversationHistoryError> {
    super::session_store::validate_session_id(session_id).map_err(|_| ConversationHistoryError)?;
    let session = super::session_store::get(session_id)
        .await
        .map_err(|_| ConversationHistoryError)?;
    super::conversation_history_resolve::from_session(
        &session,
        target,
        super::conversation_history_resolve::AttachmentKeySource::Vault,
        None,
    )
    .await
}

#[cfg(test)]
pub(crate) async fn load_for_target_with_key(
    session_id: &str,
    target: &ReplayTarget,
    key: &[u8],
) -> Result<ConversationHistory, ConversationHistoryError> {
    super::session_store::validate_session_id(session_id).map_err(|_| ConversationHistoryError)?;
    let session = super::session_store::get(session_id)
        .await
        .map_err(|_| ConversationHistoryError)?;
    super::conversation_history_resolve::from_session(
        &session,
        target,
        super::conversation_history_resolve::AttachmentKeySource::Fixed(
            key.try_into().map_err(|_| ConversationHistoryError)?,
        ),
        None,
    )
    .await
}

pub(super) async fn load_for_admission(
    session_id: &str,
    target: &ReplayTarget,
    current_user_id: &str,
    current: super::conversation_input::ResolvedTurnInput,
    key_source: super::conversation_history_resolve::AttachmentKeySource,
) -> Result<ConversationHistory, ConversationHistoryError> {
    load_for_admission_continuation(
        session_id,
        &ContinuationTarget::Replay(target.clone()),
        current_user_id,
        current,
        key_source,
    )
    .await
}

pub(super) async fn load_for_admission_continuation(
    session_id: &str,
    target: &ContinuationTarget,
    current_user_id: &str,
    current: super::conversation_input::ResolvedTurnInput,
    key_source: super::conversation_history_resolve::AttachmentKeySource,
) -> Result<ConversationHistory, ConversationHistoryError> {
    let session = super::session_store::get(session_id)
        .await
        .map_err(|_| ConversationHistoryError)?;
    let mut history = super::conversation_history_resolve::from_session_for_continuation(
        &session,
        target,
        key_source,
        Some(current_user_id),
    )
    .await?;
    history.overlay_current_input(current_user_id, current)?;
    Ok(history)
}

pub(crate) fn from_session(
    session: &super::types_session::AgentSession,
    target: &ReplayTarget,
) -> Result<ConversationHistory, ConversationHistoryError> {
    super::conversation_history_build::from_session(session, target)
}
