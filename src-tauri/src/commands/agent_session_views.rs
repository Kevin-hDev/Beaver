use crate::models::agent_session_contract::AgentSessionView;
use crate::models::compression_profile_contract::ResolvedCompressionProfileView;
use crate::services::agent_local::session_store;

#[tauri::command]
pub async fn get_agent_session(id: String) -> Result<AgentSessionView, String> {
    let session = session_store::get(&id).await?;
    let mut view = crate::services::agent_local::session_view::from_session(&session)?;
    crate::services::agent_local::session_artifact_verification::apply(&session, &mut view).await;
    Ok(view)
}

#[tauri::command]
pub async fn get_session_compression_profile(
    session_id: String,
) -> Result<ResolvedCompressionProfileView, String> {
    let session = session_store::get(&session_id).await?;
    resolved_compression_profile_view(&session).await
}

#[tauri::command]
pub async fn set_session_compression_profile(
    session_id: String,
    profile_id: String,
) -> Result<ResolvedCompressionProfileView, String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&session_id).await?;
    if profile_id != crate::services::compress::profile_defaults::BEAVER_PROFILE_ID
        && uuid::Uuid::parse_str(&profile_id).is_err()
    {
        return Err("compression_profiles_unavailable".to_string());
    }
    let document = crate::services::compress::profile_store::load_document().map_err(|error| {
        log::warn!("session_compression_profile_load_failed error={error:?}");
        "compression_profiles_unavailable".to_string()
    })?;
    if !document
        .profiles
        .iter()
        .any(|profile| profile.id == profile_id)
    {
        return Err("compression_profiles_unavailable".to_string());
    }
    crate::services::agent_local::session_ops::set_compression_profile(
        &session_id,
        crate::services::agent_local::types_session::SessionCompressionProfileSelection {
            profile_id,
            global_selection_revision: document.global_selection_revision,
        },
    )
    .await?;
    let session = session_store::get(&session_id).await?;
    resolved_compression_profile_view(&session).await
}

pub(crate) async fn resolved_compression_profile_view(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> Result<ResolvedCompressionProfileView, String> {
    let resolved = crate::services::compress::profile_resolve::resolve_for_session(session)
        .map_err(|error| {
            log::warn!("session_compression_profile_resolve_failed error={error:?}");
            "compression_profiles_unavailable".to_string()
        })?;
    let context =
        crate::services::compress::context_resolve::resolve(&session.provider, &session.model)
            .await;
    Ok(ResolvedCompressionProfileView::from_resolved(
        &resolved,
        context.configured,
    ))
}

#[tauri::command]
pub async fn create_agent_session(
    name: String,
    model: String,
    provider: Option<String>,
    project_id: Option<String>,
    reasoning_mode: Option<String>,
    supports_thinking: Option<bool>,
    fast_mode_enabled: Option<bool>,
) -> Result<AgentSessionView, String> {
    let provider = provider.unwrap_or_else(|| "ollama".to_string());
    let mut session = session_store::create_with_project_and_fast_mode(
        &name,
        &model,
        &provider,
        project_id,
        fast_mode_enabled.unwrap_or(false),
    )
    .await?;
    if reasoning_mode.is_some() {
        session_store::update_reasoning(&session.id, reasoning_mode, supports_thinking).await?;
        if let Ok(updated) = session_store::get(&session.id).await {
            session = updated;
        }
    }
    crate::services::agent_local::session_view::from_session(&session)
}
