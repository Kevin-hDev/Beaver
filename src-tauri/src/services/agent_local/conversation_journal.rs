#[path = "conversation_journal_record.rs"]
mod record;
#[path = "conversation_journal_store.rs"]
mod store;
#[path = "conversation_journal_validation.rs"]
mod validation;

use chrono::Utc;

use super::types_ollama::ChatMessage;
pub(crate) use validation::validate_tool_results;
use validation::{assistant_tool_ids, error};

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
}

struct SubagentOwner {
    run_id: String,
    execution_id: String,
}

impl ConversationJournal {
    pub(crate) fn turn_ids(&self) -> (&str, &str, &str) {
        (
            &self.turn_id,
            &self.user_message_id,
            &self.assistant_message_id,
        )
    }

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
        self.append(vec![record::from_message(
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
    ) -> Result<(), String> {
        if self.committed || self.partial || self.expected_tool_ids.is_empty() {
            return Err(error());
        }
        validate_tool_results(messages, &self.expected_tool_ids)?;
        let records = messages
            .iter()
            .map(|message| {
                record::from_message(
                    message,
                    uuid::Uuid::new_v4().to_string(),
                    &self.turn_id,
                    &self.request_id,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.append(records).await?;
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
        self.append(vec![record::from_message(
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
        self.update(move |session| {
            if session.messages.len().saturating_add(records.len())
                > super::session_limits::MAX_MESSAGES_PER_SESSION
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
