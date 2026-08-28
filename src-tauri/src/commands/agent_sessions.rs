use crate::models::agent_session_contract::{EditUserMessageInput, SessionMetadataPatch};
use crate::services::agent_local::session_store;
use crate::services::agent_local::types_session::{AgentSessionMeta, PreserveReasoningSetting};

#[tauri::command]
pub async fn list_agent_sessions() -> Result<Vec<AgentSessionMeta>, String> {
    session_store::list().await
}

/// Range une liste de conversations dans l'ordre reçu. `project_id` absent
/// désigne les conversations qui n'appartiennent à aucun projet.
#[tauri::command]
pub async fn reorder_agent_sessions(
    project_id: Option<String>,
    ids: Vec<String>,
) -> Result<(), String> {
    crate::services::agent_local::session_order::set(project_id.as_deref(), ids).await
}

/// Range la liste des conversations épinglées dans l'ordre reçu. Les noms
/// de listes restent dans session_order : rien ne traverse la frontière.
#[tauri::command]
pub async fn reorder_pinned_agent_sessions(ids: Vec<String>) -> Result<(), String> {
    crate::services::agent_local::session_order::set_pinned(ids).await
}

#[tauri::command]
pub async fn list_archived_agent_sessions() -> Result<Vec<AgentSessionMeta>, String> {
    session_store::list_archived().await
}

#[tauri::command]
pub async fn update_session_project(id: String, project_id: String) -> Result<(), String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    crate::services::agent_local::directory_access::project_path(&project_id).await?;
    crate::services::agent_local::session_ops::apply_metadata_patch(
        &id,
        SessionMetadataPatch {
            project_id: Some(project_id),
            ..SessionMetadataPatch::default()
        },
    )
    .await
}

#[tauri::command]
pub async fn get_session_permission_state(
    id: String,
) -> Result<crate::services::agent_local::session_permission_state::SessionPermissionState, String>
{
    crate::services::agent_local::session_permission_state::load(&id).await
}

#[tauri::command]
pub async fn set_session_permission_mode(
    id: String,
    mode: String,
) -> Result<crate::services::agent_local::session_permission_state::SessionPermissionState, String>
{
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    let mode =
        crate::services::agent_local::session_permission_state::PermissionMode::parse(&mode)?;
    crate::services::agent_local::session_permission_state::set_mode(&id, mode).await
}

#[tauri::command]
pub async fn prepare_agent_send(
    id: String,
    working_dir: Option<String>,
) -> Result<crate::services::agent_local::agent_send_preflight::PrepareAgentSend, String> {
    crate::services::agent_local::agent_send_preflight::prepare(&id, working_dir.as_deref()).await
}

#[tauri::command]
pub async fn resolve_missing_session_directory(
    id: String,
    missing_path: String,
    action: crate::services::agent_local::agent_send_preflight::MissingDirectoryAction,
) -> Result<String, String> {
    crate::services::agent_local::agent_send_preflight::resolve(&id, &missing_path, action).await
}

#[tauri::command]
pub async fn update_session_metadata(
    id: String,
    patch: SessionMetadataPatch,
) -> Result<(), String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    if let Some(project_id) = patch.project_id.as_deref() {
        crate::services::agent_local::directory_access::project_path(project_id).await?;
    }
    crate::services::agent_local::session_ops::apply_metadata_patch(&id, patch).await
}

#[tauri::command]
pub async fn set_session_fast_mode(id: String, enabled: bool) -> Result<bool, String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    session_store::update_fast_mode(&id, enabled).await
}

#[tauri::command]
pub async fn rename_agent_session(id: String, name: String) -> Result<(), String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    session_store::rename(&id, &name).await
}

#[tauri::command]
pub async fn update_session_model(
    id: String,
    model: String,
    provider: String,
    reasoning_mode: Option<String>,
    supports_thinking: Option<bool>,
) -> Result<(), String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    session_store::update_model(&id, &model, &provider, reasoning_mode, supports_thinking).await
}

#[tauri::command]
pub async fn update_session_reasoning(
    id: String,
    reasoning_mode: Option<String>,
    supports_thinking: Option<bool>,
) -> Result<(), String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    session_store::update_reasoning(&id, reasoning_mode, supports_thinking).await
}

#[tauri::command]
pub async fn update_session_continuity(id: String, setting: String) -> Result<(), String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    let setting = parse_continuity_setting(&setting)?;
    let session = session_store::get(&id).await?;
    let capability = crate::services::agent_local::session_view::continuity_capability(&session)
        .ok_or_else(invalid_continuity_update)?;
    let allowed = match setting {
        PreserveReasoningSetting::Off => {
            capability.requirement
                == crate::models::agent_session_contract::ContinuityRequirement::Optional
        }
        PreserveReasoningSetting::Local => capability.local_available,
        PreserveReasoningSetting::Remote => capability.remote_available,
    };
    if !allowed {
        return Err(invalid_continuity_update());
    }
    crate::services::agent_local::session_continuity::update(&id, setting).await
}

fn invalid_continuity_update() -> String {
    "Modification de session impossible".to_string()
}

fn parse_continuity_setting(value: &str) -> Result<PreserveReasoningSetting, String> {
    match value {
        "off" => Ok(PreserveReasoningSetting::Off),
        "local" => Ok(PreserveReasoningSetting::Local),
        "remote" => Ok(PreserveReasoningSetting::Remote),
        _ => Err(invalid_continuity_update()),
    }
}

#[tauri::command]
pub async fn set_session_plan_mode(id: String, enabled: bool) -> Result<(), String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    crate::services::agent_local::tool_plan::set_enabled(&id, enabled).await
}

#[tauri::command]
pub async fn delete_agent_session(id: String) -> Result<(), String> {
    // Supprimer, archiver ou restaurer agit sur le cycle de vie de toute la famille ;
    // ce n'est pas une édition du contenu produit par le runtime du sous-agent.
    session_store::delete(&id).await
}

#[tauri::command]
pub async fn archive_agent_session(id: String) -> Result<(), String> {
    if session_store::get(&id)
        .await
        .is_ok_and(|session| session.parent_session_id.is_some())
    {
        return match crate::services::agent_local::subagent_archive::archive(&id).await {
            Ok(crate::services::agent_local::subagent_archive::ArchiveOutcome::Archived) => Ok(()),
            Ok(_) => Err("Impossible d'archiver cette session.".to_string()),
            Err(_) => Err("Impossible d'archiver cette session.".to_string()),
        };
    }
    session_store::archive(&id).await
}

#[tauri::command]
pub async fn restore_agent_session(id: String) -> Result<(), String> {
    session_store::restore(&id).await
}

#[tauri::command]
pub async fn pin_agent_session(id: String) -> Result<(), String> {
    crate::services::agent_local::session_pin::pin(&id).await
}

#[tauri::command]
pub async fn unpin_agent_session(id: String) -> Result<(), String> {
    crate::services::agent_local::session_pin::unpin(&id).await
}

#[tauri::command]
pub async fn export_agent_session_markdown(id: String) -> Result<String, String> {
    session_store::export_markdown(&id).await
}

#[tauri::command]
pub async fn truncate_and_replace_at(
    session_id: String,
    input: EditUserMessageInput,
) -> Result<(), String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&session_id).await?;
    crate::services::agent_local::session_ops::edit_user_message(&session_id, input).await
}
