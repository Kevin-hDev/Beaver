use crate::models::agent_session_contract::EditUserMessageInput;
#[cfg(test)]
use crate::services::reasoning_continuity::contract::ReplayTarget;

#[cfg(test)]
use super::conversation_admission::{error, ConversationAdmissionError};
#[cfg(test)]
use super::conversation_history::ConversationHistory;
use super::types_session::AgentSession;

#[cfg(test)]
pub async fn edit_user_message(
    session_id: &str,
    input: EditUserMessageInput,
    target: &ReplayTarget,
) -> Result<ConversationHistory, ConversationAdmissionError> {
    edit_inner(
        session_id,
        input,
        target,
        super::conversation_history_resolve::AttachmentKeySource::Vault,
        || async {},
        |prepared| async move { super::session_store::save_prepared(prepared).await },
    )
    .await
}

#[cfg(test)]
async fn edit_inner<A, AFut, W, WFut>(
    session_id: &str,
    input: EditUserMessageInput,
    target: &ReplayTarget,
    key_source: super::conversation_history_resolve::AttachmentKeySource,
    after_preflight: A,
    writer: W,
) -> Result<ConversationHistory, ConversationAdmissionError>
where
    A: FnOnce() -> AFut,
    AFut: std::future::Future<Output = ()>,
    W: FnOnce(super::session_store_document::PreparedSessionDocument) -> WFut,
    WFut: std::future::Future<Output = Result<(), String>>,
{
    super::session_store::validate_session_id(session_id).map_err(|_| error())?;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id)
        .await
        .map_err(|_| error())?;
    // Resolve every local authority before the durable mutation. Otherwise a
    // missing historical grant/file/skill would be discovered only after save.
    super::conversation_history_resolve::from_session(&session, target, key_source, None)
        .await
        .map_err(|_| error())?;
    after_preflight().await;
    apply_to_session(&mut session, input).map_err(|_| error())?;
    session.messages.last_mut().ok_or_else(error)?.replay_source =
        Some(crate::services::reasoning_continuity::envelope::ReasoningSource::from_target(target));
    let prepared = super::session_store_document::prepare(&session)
        .await
        .map_err(|_| error())?;
    let history = super::conversation_history_resolve::from_session(
        prepared.session(),
        target,
        key_source,
        None,
    )
    .await
    .map_err(|_| error())?;
    writer(prepared).await.map_err(|_| error())?;
    Ok(history)
}

pub(super) fn apply_to_session(
    session: &mut AgentSession,
    input: EditUserMessageInput,
) -> Result<(), String> {
    if super::session_migration_ids::validate_id(&input.message_id).is_err()
        || input.new_content.len() > crate::models::agent_turn_contract::MAX_TURN_CONTENT_BYTES
        || input.new_content.contains('\0')
    {
        return Err("Modification de session impossible".to_string());
    }
    let index = session
        .messages
        .iter()
        .position(|message| message.id == input.message_id && message.role == "user")
        .ok_or_else(|| "Modification de session impossible".to_string())?;
    session.messages.truncate(index + 1);
    session.messages[index].content = input.new_content;
    session.context_tokens = None;
    super::session_store_messages::recompute_accumulated_tokens(session);
    Ok(())
}

#[cfg(test)]
pub(crate) async fn edit_user_message_with_writer<W, Fut>(
    session_id: &str,
    input: EditUserMessageInput,
    target: &ReplayTarget,
    writer: W,
) -> Result<ConversationHistory, ConversationAdmissionError>
where
    W: FnOnce(super::session_store_document::PreparedSessionDocument) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    edit_inner(
        session_id,
        input,
        target,
        super::conversation_history_resolve::AttachmentKeySource::Vault,
        || async {},
        writer,
    )
    .await
}

#[cfg(test)]
pub(crate) async fn edit_user_message_after_preflight_with_key_and_writer<A, AFut, W, WFut>(
    session_id: &str,
    input: EditUserMessageInput,
    target: &ReplayTarget,
    key: &[u8],
    after_preflight: A,
    writer: W,
) -> Result<ConversationHistory, ConversationAdmissionError>
where
    A: FnOnce() -> AFut,
    AFut: std::future::Future<Output = ()>,
    W: FnOnce(super::session_store_document::PreparedSessionDocument) -> WFut,
    WFut: std::future::Future<Output = Result<(), String>>,
{
    edit_inner(
        session_id,
        input,
        target,
        super::conversation_history_resolve::AttachmentKeySource::Fixed(
            key.try_into().map_err(|_| error())?,
        ),
        after_preflight,
        writer,
    )
    .await
}
