use std::collections::HashSet;

use uuid::Uuid;

use crate::models::agent_turn_contract::ResumeTurnInput;
use crate::services::reasoning_continuity::contract::{ContinuationTarget, ReplayTarget};

use super::conversation_admission::{error, unique_uuid, AdmittedTurn, ConversationAdmissionError};
use super::conversation_history::ProviderRole;
use super::conversation_history_resolve::AttachmentKeySource;

pub async fn resume(
    session_id: &str,
    input: ResumeTurnInput,
    target: ReplayTarget,
) -> Result<AdmittedTurn, ConversationAdmissionError> {
    resume_for_continuation(session_id, input, ContinuationTarget::Replay(target)).await
}

pub async fn resume_for_continuation(
    session_id: &str,
    input: ResumeTurnInput,
    target: ContinuationTarget,
) -> Result<AdmittedTurn, ConversationAdmissionError> {
    let lease = super::session_locks::acquire_admission_lease(session_id).await;
    resume_with_lease(&lease, input, target).await
}

pub(crate) async fn resume_with_lease(
    lease: &super::session_locks::AdmissionLease,
    input: ResumeTurnInput,
    target: ContinuationTarget,
) -> Result<AdmittedTurn, ConversationAdmissionError> {
    resume_inner(
        lease.session_id(),
        input,
        target,
        AttachmentKeySource::Vault,
    )
    .await
}

async fn resume_inner(
    session_id: &str,
    input: ResumeTurnInput,
    target: ContinuationTarget,
    key_source: AttachmentKeySource,
) -> Result<AdmittedTurn, ConversationAdmissionError> {
    super::session_store::validate_session_id(session_id).map_err(|_| error())?;
    let session = super::session_store::get(session_id)
        .await
        .map_err(|_| error())?;
    let history = super::conversation_history_resolve::from_session_for_continuation(
        &session,
        &target,
        key_source,
        None,
    )
    .await
    .map_err(|_| error())?;
    let message = session.messages.last().ok_or_else(error)?;
    let provider = history.messages.last().ok_or_else(error)?;
    if message.id != input.message_id
        || message.role != "user"
        || provider.role != ProviderRole::User
        || !message.files.is_empty()
        || message.skill_names.as_ref().is_some_and(|names| !names.is_empty())
    {
        return Err(error());
    }
    let mut used = session
        .messages
        .iter()
        .flat_map(|message| [message.id.clone(), message.turn_id.clone()])
        .collect::<HashSet<_>>();
    let assistant_message_id = unique_uuid(&mut used, &mut || Uuid::new_v4().to_string())?;
    Ok(AdmittedTurn {
        turn_id: message.turn_id.clone(),
        user_message_id: message.id.clone(),
        assistant_message_id,
        history,
    })
}

#[cfg(test)]
pub(crate) async fn resume_with_key(
    session_id: &str,
    input: ResumeTurnInput,
    target: ReplayTarget,
    key: &[u8],
) -> Result<AdmittedTurn, ConversationAdmissionError> {
    let lease = super::session_locks::acquire_admission_lease(session_id).await;
    resume_inner(
        lease.session_id(),
        input,
        ContinuationTarget::Replay(target),
        AttachmentKeySource::Fixed(key.try_into().map_err(|_| error())?),
    )
    .await
}
