use crate::models::agent_turn_contract::{NewUserTurnInput, SkillReference, TurnAttachmentInput};
use crate::services::reasoning_continuity::contract::ContinuationTarget;
#[cfg(test)]
use crate::services::reasoning_continuity::contract::ReplayTarget;

use super::conversation_history::{ConversationHistory, ConversationHistoryError};
use super::types_message::AgentMessage;

#[derive(Clone, Copy)]
pub(super) enum AttachmentKeySource {
    Vault,
    #[cfg(test)]
    Fixed([u8; 32]),
}

#[cfg(test)]
pub(super) async fn from_session(
    session: &super::types_session::AgentSession,
    target: &ReplayTarget,
    key_source: AttachmentKeySource,
    skip_user_id: Option<&str>,
) -> Result<ConversationHistory, ConversationHistoryError> {
    from_session_for_continuation(
        session,
        &ContinuationTarget::Replay(target.clone()),
        key_source,
        skip_user_id,
    )
    .await
}

pub(super) async fn from_session_for_continuation(
    session: &super::types_session::AgentSession,
    target: &ContinuationTarget,
    key_source: AttachmentKeySource,
    skip_user_id: Option<&str>,
) -> Result<ConversationHistory, ConversationHistoryError> {
    let mut history = super::conversation_history_build::from_continuation(session, target)?;
    let needs_key = session.messages.iter().any(|message| {
        message.role == "user"
            && Some(message.id.as_str()) != skip_user_id
            && message.files.iter().any(|file| !file.path.is_empty())
    });
    let key = needs_key
        .then(|| load_key(key_source))
        .transpose()
        .ok()
        .flatten();
    for message in session
        .messages
        .iter()
        .rev()
        .filter(|message| message.role == "user" && Some(message.id.as_str()) != skip_user_id)
    {
        let has_skill_ids = message
            .skill_ids
            .as_ref()
            .is_some_and(|ids| !ids.is_empty());
        if message.files.is_empty() && !has_skill_ids {
            continue;
        }
        let resolved = super::conversation_input::resolve_persisted_with_key(
            persisted_input(message)?,
            key.as_ref().map_or(&[], |value| value.as_slice()),
        )
        .await
        .map_err(|_| ConversationHistoryError)?;
        history.overlay_resolved_input(&message.id, resolved)?;
    }
    Ok(history)
}

fn load_key(
    source: AttachmentKeySource,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ConversationHistoryError> {
    match source {
        AttachmentKeySource::Vault => crate::services::attachment_access::attachment_key()
            .map_err(|_| ConversationHistoryError),
        #[cfg(test)]
        AttachmentKeySource::Fixed(key) => Ok(zeroize::Zeroizing::new(key.to_vec())),
    }
}

fn persisted_input(message: &AgentMessage) -> Result<NewUserTurnInput, ConversationHistoryError> {
    let skills = match (&message.skill_ids, &message.skill_names) {
        (Some(ids), Some(names)) if ids.len() == names.len() => ids
            .iter()
            .zip(names)
            .map(|(id, name)| SkillReference {
                id: id.clone(),
                name: Some(name.clone()),
            })
            .collect(),
        (None, _) => Vec::new(),
        _ => return Err(ConversationHistoryError),
    };
    let files = message
        .files
        .iter()
        .map(|file| TurnAttachmentInput {
            name: file.name.clone(),
            path: file.path.clone(),
            mime_type: file.mime_type.clone(),
            size: file.size,
            thumbnail: file.thumbnail.clone(),
            access_grant: file.access_grant.clone(),
        })
        .collect();
    Ok(NewUserTurnInput {
        content: message.content.clone(),
        files,
        skills,
    })
}
