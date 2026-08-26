#![allow(dead_code, reason = "the Rust chat boundary adopts admission in Task 9")]

use std::collections::HashSet;
use std::fmt;

use chrono::Utc;
use uuid::Uuid;

use crate::models::agent_turn_contract::ResumeTurnInput;
use crate::services::reasoning_continuity::contract::ReplayTarget;

use super::conversation_history::{ConversationHistory, ProviderRole};
use super::conversation_input::ResolvedTurnInput;
use super::types_message::AgentMessage;
use super::types_session::AgentSession;

pub const PUBLIC_ERROR_CODE: &str = super::conversation_history::PUBLIC_ERROR_CODE;
#[cfg(test)]
pub(crate) use super::conversation_edit::{
    edit_user_message, edit_user_message_after_preflight_with_key_and_writer,
    edit_user_message_with_writer,
};
#[cfg(test)]
pub(crate) use super::conversation_resume::resume_with_key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversationAdmissionError;

impl fmt::Display for ConversationAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(PUBLIC_ERROR_CODE)
    }
}

impl std::error::Error for ConversationAdmissionError {}

#[derive(Debug)]
pub struct AdmittedTurn {
    pub turn_id: String,
    pub user_message_id: String,
    pub assistant_message_id: String,
    pub history: ConversationHistory,
}

pub async fn resume(
    session_id: &str,
    input: ResumeTurnInput,
    target: ReplayTarget,
) -> Result<AdmittedTurn, ConversationAdmissionError> {
    super::conversation_resume::resume(session_id, input, target).await
}

pub async fn new_turn(
    session_id: &str,
    input: ResolvedTurnInput,
    target: ReplayTarget,
) -> Result<AdmittedTurn, ConversationAdmissionError> {
    new_turn_inner(
        session_id,
        input,
        target,
        super::conversation_history_resolve::AttachmentKeySource::Vault,
        || async {},
        |session| async move { super::session_store::save(&session).await },
        || async {},
    )
    .await
}

async fn new_turn_inner<A, AFut, W, WFut, P, PFut>(
    session_id: &str,
    input: ResolvedTurnInput,
    target: ReplayTarget,
    key_source: super::conversation_history_resolve::AttachmentKeySource,
    after_load: A,
    writer: W,
    after_persist: P,
) -> Result<AdmittedTurn, ConversationAdmissionError>
where
    A: FnOnce() -> AFut,
    AFut: std::future::Future<Output = ()>,
    W: FnOnce(AgentSession) -> WFut,
    WFut: std::future::Future<Output = Result<(), String>>,
    P: FnOnce() -> PFut,
    PFut: std::future::Future<Output = ()>,
{
    super::session_store::validate_session_id(session_id).map_err(|_| error())?;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id)
        .await
        .map_err(|_| error())?;
    after_load().await;
    let history = super::conversation_history_resolve::from_session(
        &session,
        &target,
        key_source,
        None,
    )
    .await
    .map_err(|_| error())?;
    if history.messages.last().is_some_and(|message| message.role == ProviderRole::User)
        || session.messages.len() >= super::session_limits::MAX_MESSAGES_PER_SESSION
    {
        return Err(error());
    }

    let mut used = session
        .messages
        .iter()
        .flat_map(|message| [message.id.clone(), message.turn_id.clone()])
        .collect::<HashSet<_>>();
    let (turn_id, user_message_id, assistant_message_id) =
        allocate_ids(&mut used, || Uuid::new_v4().to_string())?;
    let skill_names = (!input.skills.is_empty())
        .then(|| input.skills.iter().map(|skill| skill.name.clone()).collect());
    let skill_ids = (!input.skills.is_empty())
        .then(|| input.skills.iter().map(|skill| skill.id.clone()).collect());
    let message = AgentMessage {
        id: user_message_id.clone(),
        turn_id: turn_id.clone(),
        role: "user".into(),
        content: input.user_content.clone(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: Some(
            crate::services::reasoning_continuity::envelope::ReasoningSource::from_target(&target),
        ),
        tool_activities: None,
        segments: None,
        files: input.files.clone(),
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names,
        skill_ids,
        stream_run_id: None,
        stream_part: None,
    };
    let housekeeping = super::session_store_todos::apply_user_turn(&mut session, true);
    session.messages.push(message);
    session.updated_at = Some(Utc::now());
    super::session_store_messages::recompute_accumulated_tokens(&mut session);
    writer(session).await.map_err(|_| error())?;

    let history = super::conversation_history::load_for_admission(
        session_id,
        &target,
        &user_message_id,
        input,
        key_source,
    )
    .await
    .map_err(|_| error())?;
    after_persist().await;
    if housekeeping.should_emit_empty_update {
        super::tool_todo::emit_update(session_id, Vec::new());
    }
    Ok(AdmittedTurn {
        turn_id,
        user_message_id,
        assistant_message_id,
        history,
    })
}

pub(super) fn unique_uuid<F>(
    used: &mut HashSet<String>,
    generator: &mut F,
) -> Result<String, ConversationAdmissionError>
where
    F: FnMut() -> String,
{
    for _ in 0..4 {
        let candidate = generator();
        let valid = Uuid::parse_str(&candidate)
            .ok()
            .is_some_and(|id| id.get_version_num() == 4);
        if valid && used.insert(candidate.clone()) {
            return Ok(candidate);
        }
    }
    Err(error())
}

fn allocate_ids<F>(
    used: &mut HashSet<String>,
    mut generator: F,
) -> Result<(String, String, String), ConversationAdmissionError>
where
    F: FnMut() -> String,
{
    Ok((
        unique_uuid(used, &mut generator)?,
        unique_uuid(used, &mut generator)?,
        unique_uuid(used, &mut generator)?,
    ))
}

pub(super) const fn error() -> ConversationAdmissionError {
    ConversationAdmissionError
}

#[cfg(test)]
include!("conversation_admission_test_seams.rs");
