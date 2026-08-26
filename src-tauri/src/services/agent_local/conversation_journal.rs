#[path = "conversation_journal_record.rs"]
mod record;

use chrono::Utc;
use std::collections::HashSet;

use super::types_ollama::ChatMessage;

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
        (&self.turn_id, &self.user_message_id, &self.assistant_message_id)
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
        for id in [&turn_id, &user_message_id, &assistant_message_id, &request_id] {
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

    pub(crate) async fn persist_assistant_step(&mut self, message: &ChatMessage) -> Result<(), String> {
        if self.committed || self.partial || message.role != "assistant" { return Err(error()); }
        let ids = assistant_tool_ids(message)?;
        let message_id = if self.assistant_steps == 0 { self.assistant_message_id.clone() } else { uuid::Uuid::new_v4().to_string() };
        self.append(vec![record::from_message(message, message_id, &self.turn_id, &self.request_id)?]).await?;
        self.expected_tool_ids = ids;
        self.assistant_steps += 1;
        Ok(())
    }

    pub(crate) async fn persist_tool_results(&mut self, messages: &[ChatMessage]) -> Result<(), String> {
        if self.committed || self.partial || self.expected_tool_ids.is_empty() { return Err(error()); }
        validate_tool_results(messages, &self.expected_tool_ids)?;
        let records = messages.iter().map(|message| record::from_message(message, uuid::Uuid::new_v4().to_string(), &self.turn_id, &self.request_id)).collect::<Result<Vec<_>, _>>()?;
        self.append(records).await?;
        self.expected_tool_ids.clear();
        Ok(())
    }

    pub(crate) async fn persist_partial(&mut self, mut message: ChatMessage) -> Result<(), String> {
        if self.committed || self.partial || message.role != "assistant" {
            return Err(error());
        }
        if let Some(envelope) = &mut message.continuation {
            envelope.completion = crate::services::reasoning_continuity::envelope::CompletionState::Partial;
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

    pub(crate) async fn commit_turn(&mut self) -> Result<(), String> {
        if self.committed || self.partial || self.assistant_steps == 0 || !self.expected_tool_ids.is_empty() { return Err(error()); }
        let run_id = self.request_id.clone();
        self.update(move |session| {
            let mut found = false;
            for message in &mut session.messages {
                if message.stream_run_id.as_deref() == Some(&run_id) {
                    message.stream_part = Some("final".to_string());
                    found = true;
                }
            }
            found.then_some(()).ok_or_else(error)
        }).await?;
        self.committed = true;
        Ok(())
    }

    async fn append(&self, records: Vec<super::types_message::AgentMessage>) -> Result<(), String> {
        if records.is_empty() { return Err(error()); }
        self.update(move |session| {
            if session.messages.len().saturating_add(records.len()) > super::session_limits::MAX_MESSAGES_PER_SESSION { return Err(error()); }
            session.messages.extend(records);
            session.updated_at = Some(Utc::now());
            super::session_store_messages::recompute_accumulated_tokens(session);
            Ok(())
        }).await
    }

    async fn update<F>(&self, update: F) -> Result<(), String>
    where F: FnOnce(&mut super::types_session::AgentSession) -> Result<(), String> {
        self.verify_subagent_owner().await?;
        let lock = super::session_store::lock_session(&self.session_id).await;
        let _guard = lock.lock().await;
        let mut session = super::session_store::get(&self.session_id).await.map_err(|_| error())?;
        if self
            .subagent_owner
            .as_ref()
            .is_some_and(|owner| session.subagent_run_id.as_deref() != Some(&owner.run_id))
        {
            return Err(error());
        }
        update(&mut session)?;
        super::session_store::save(&session).await.map_err(|_| error())
    }

    async fn verify_subagent_owner(&self) -> Result<(), String> {
        let Some(owner) = &self.subagent_owner else {
            return Ok(());
        };
        super::subagent_registry::owns_execution(
            &self.session_id,
            &owner.run_id,
            &owner.execution_id,
        )
        .await
        .then_some(())
        .ok_or_else(error)
    }
}

pub(crate) fn validate_tool_results(messages: &[ChatMessage], expected: &[String]) -> Result<(), String> {
    if messages.iter().any(|message| message.role != "tool") {
        return Err(error());
    }
    let actual = messages.iter().filter(|message| message.role == "tool").map(|message| message.tool_call_id.clone().ok_or_else(error)).collect::<Result<Vec<_>, _>>()?;
    if actual != expected || actual.iter().collect::<HashSet<_>>().len() != actual.len() { return Err(error()); }
    Ok(())
}

fn assistant_tool_ids(message: &ChatMessage) -> Result<Vec<String>, String> {
    let ids = message.tool_calls.as_deref().unwrap_or_default().iter().map(|call| call.id.clone().ok_or_else(error)).collect::<Result<Vec<_>, _>>()?;
    (ids.iter().collect::<HashSet<_>>().len() == ids.len()).then_some(ids).ok_or_else(error)
}

fn error() -> String { "conversation_journal_failed".to_string() }
