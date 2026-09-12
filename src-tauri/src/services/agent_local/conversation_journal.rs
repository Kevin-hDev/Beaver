#[path = "conversation_journal_record.rs"]
mod record;
#[path = "conversation_journal_store.rs"]
mod store;
#[path = "conversation_journal_context.rs"]
mod context;
#[path = "conversation_journal_validation.rs"]
mod validation;
#[path = "conversation_journal_recovery.rs"]
mod recovery;

use chrono::Utc;

use super::types_ollama::ChatMessage;
pub(crate) use validation::validate_tool_results;
use validation::{assistant_tool_ids, capacity_error, error};

/// Unique owner of durable provider checkpoints for one admitted turn.
pub(crate) struct ConversationJournal {
    session_id: String,
    turn_id: String,
    user_message_id: String,
    assistant_message_id: String,
    request_id: String,
    expected_tool_ids: Vec<String>,
    assistant_steps: usize,
    subagent_owner: Option<SubagentOwner>,
    partial: bool,
    committed: bool,
    recovery_log: Option<super::stream_recovery_log::StreamRecoveryLog>,
    recovery_owner: Option<super::stream_recovery_owners::OwnerLease>,
}

struct SubagentOwner {
    run_id: String,
    execution_id: String,
}

impl ConversationJournal {
    pub(crate) fn new(
        session_id: String,
        turn_id: String,
        user_message_id: String,
        assistant_message_id: String,
        request_id: String,
    ) -> Result<Self, String> {
        Self::new_inner(
            session_id,
            turn_id,
            user_message_id,
            assistant_message_id,
            request_id,
            None,
        )
    }

    pub(crate) fn new_for_subagent(
        session_id: String,
        turn_id: String,
        user_message_id: String,
        assistant_message_id: String,
        request_id: String,
        run_id: String,
        execution_id: String,
    ) -> Result<Self, String> {
        for id in [&run_id, &execution_id] {
            uuid::Uuid::parse_str(id).map_err(|_| error())?;
        }
        Self::new_inner(
            session_id,
            turn_id,
            user_message_id,
            assistant_message_id,
            request_id,
            Some(SubagentOwner {
                run_id,
                execution_id,
            }),
        )
    }

    fn new_inner(
        session_id: String,
        turn_id: String,
        user_message_id: String,
        assistant_message_id: String,
        request_id: String,
        subagent_owner: Option<SubagentOwner>,
    ) -> Result<Self, String> {
        super::session_store::validate_session_id(&session_id)?;
        for id in [
            &turn_id,
            &user_message_id,
            &assistant_message_id,
            &request_id,
        ] {
            uuid::Uuid::parse_str(id).map_err(|_| error())?;
        }
        Ok(Self {
            session_id,
            turn_id,
            user_message_id,
            assistant_message_id,
            request_id,
            expected_tool_ids: Vec::new(),
            assistant_steps: 0,
            subagent_owner,
            partial: false,
            committed: false,
            recovery_log: None,
            recovery_owner: None,
        })
    }

    pub(crate) async fn persist_assistant_step(
        &mut self,
        message: &ChatMessage,
    ) -> Result<(), String> {
        if self.committed || self.partial || message.role != "assistant" {
            return Err(error());
        }
        let ids = assistant_tool_ids(message)?;
        let message_id = if self.assistant_steps == 0 {
            self.assistant_message_id.clone()
        } else {
            uuid::Uuid::new_v4().to_string()
        };
        self.append_staged(vec![record::from_message(
            message,
            message_id,
            &self.turn_id,
            &self.request_id,
        )?])
        .await?;
        self.expected_tool_ids = ids;
        self.assistant_steps += 1;
        Ok(())
    }

    pub(crate) async fn persist_tool_results(
        &mut self,
        messages: &[ChatMessage],
        artifacts: &[super::tool_execution_artifacts::AttributedArtifact],
    ) -> Result<(), String> {
        if self.committed || self.partial || self.expected_tool_ids.is_empty() {
            return Err(error());
        }
        validate_tool_results(messages, &self.expected_tool_ids)?;
        let artifacts = record::artifact_records(messages, artifacts)?;
        let records = messages
            .iter()
            .zip(artifacts)
            .map(|(message, artifacts)| {
                let mut record = record::from_message(
                    message,
                    uuid::Uuid::new_v4().to_string(),
                    &self.turn_id,
                    &self.request_id,
                )?;
                record.tool_activities = (!artifacts.is_empty()).then(|| {
                    vec![super::types_message::ToolActivityRecord::artifact_carrier(
                        message
                            .tool_name
                            .clone()
                            .unwrap_or_else(|| "tool".to_string()),
                        artifacts,
                    )]
                });
                Ok::<_, String>(record)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.append_staged(records).await?;
        self.expected_tool_ids.clear();
        Ok(())
    }

    pub(crate) async fn persist_partial(&mut self, mut message: ChatMessage) -> Result<(), String> {
        if self.committed || self.partial || message.role != "assistant" {
            return Err(error());
        }
        if let Some(envelope) = &mut message.continuation {
            envelope.completion =
                crate::services::reasoning_continuity::envelope::CompletionState::Partial;
        }
        self.append_staged(vec![record::from_message(
            &message,
            uuid::Uuid::new_v4().to_string(),
            &self.turn_id,
            &self.request_id,
        )?])
        .await?;
        self.partial = true;
        Ok(())
    }

    async fn append(&self, records: Vec<super::types_message::AgentMessage>) -> Result<(), String> {
        if records.is_empty() {
            return Err(error());
        }
        let turn_id = self.turn_id.clone();
        let fence_turn = self.subagent_owner.is_none();
        self.update(move |session| {
            if session.messages.len().saturating_add(records.len())
                > super::session_limits::MAX_MESSAGES_PER_SESSION
            {
                return Err(capacity_error());
            }
            if fence_turn
                && session
                    .messages
                    .last()
                    .is_some_and(|message| message.turn_id != turn_id)
            {
                return Err(error());
            }
            session.messages.extend(records);
            session.updated_at = Some(Utc::now());
            super::session_store_messages::recompute_accumulated_tokens(session);
            Ok(())
        })
        .await
    }
}
