use crate::models::agent_session_contract::{EditUserMessageInput, SessionMetadataPatch};

const MAX_SESSION_NAME_BYTES: usize = 512;
const MAX_PROVIDER_ID_BYTES: usize = 64;

pub async fn apply_metadata_patch(id: &str, patch: SessionMetadataPatch) -> Result<(), String> {
    validate_patch(&patch)?;
    super::session_store_updates::update_locked(id, move |session| {
        if let Some(name) = patch.name {
            session.name = name;
        }
        if let Some(model) = patch.model {
            session.model = model;
            session.context_tokens = None;
        }
        if let Some(provider) = patch.provider {
            session.provider = provider;
            session.context_tokens = None;
        }
        if let Some(mode) = patch.reasoning_mode {
            session.thinking_enabled = mode != "off";
            session.reasoning_mode = Some(mode);
            session.context_tokens = None;
        }
        if let Some(enabled) = patch.fast_mode_enabled {
            session.fast_mode_enabled = enabled;
        }
        if let Some(project_id) = patch.project_id {
            session.project_id = Some(project_id);
        }
    })
    .await
}

pub async fn edit_user_message(id: &str, input: EditUserMessageInput) -> Result<(), String> {
    super::session_store::validate_session_id(id)?;
    let lock = super::session_store::lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(id).await?;
    super::conversation_edit::apply_to_session(&mut session, input)?;
    super::session_store::save(&session).await
}

pub async fn set_compression_profile(
    id: &str,
    selection: super::types_session::SessionCompressionProfileSelection,
) -> Result<(), String> {
    super::session_store_updates::update_locked(id, move |session| {
        session.compression_profile_selection = Some(selection);
    })
    .await
}

fn validate_patch(patch: &SessionMetadataPatch) -> Result<(), String> {
    if let Some(name) = patch.name.as_deref() {
        if name.trim().is_empty()
            || name.len() > MAX_SESSION_NAME_BYTES
            || name.chars().any(char::is_control)
        {
            return Err(invalid_mutation());
        }
    }
    if patch.model.as_deref().is_some_and(|model| {
        crate::services::reasoning_continuity::limits::validate_model_id(model).is_err()
    }) || patch
        .provider
        .as_deref()
        .is_some_and(|provider| !valid_provider(provider))
        || patch.reasoning_mode.as_ref().is_some_and(|mode| {
            crate::services::reasoning::sanitize_mode(Some(mode.clone())).as_ref() != Some(mode)
        })
        || patch.project_id.as_deref().is_some_and(|project_id| {
            super::session_migration_ids::validate_id(project_id).is_err()
        })
    {
        return Err(invalid_mutation());
    }
    Ok(())
}

fn valid_provider(provider: &str) -> bool {
    !provider.is_empty()
        && provider.len() <= MAX_PROVIDER_ID_BYTES
        && provider
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn invalid_mutation() -> String {
    "Modification de session impossible".to_string()
}
