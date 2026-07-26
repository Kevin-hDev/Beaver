use crate::services::agent_local::session_store;
use crate::services::agent_local::types_session::{AgentMessage, AgentSession, AgentSessionMeta};

#[tauri::command]
pub async fn list_agent_sessions() -> Result<Vec<AgentSessionMeta>, String> {
    session_store::list().await
}

#[tauri::command]
pub async fn list_archived_agent_sessions() -> Result<Vec<AgentSessionMeta>, String> {
    session_store::list_archived().await
}

#[tauri::command]
pub async fn get_agent_session(id: String) -> Result<AgentSession, String> {
    session_store::get(&id).await
}

#[tauri::command]
pub async fn save_agent_session(session: AgentSession) -> Result<(), String> {
    session_store::save(&session).await
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
pub async fn add_messages_to_session(
    id: String,
    messages: Vec<AgentMessage>,
    tokens: u32,
) -> Result<(), String> {
    session_store::add_messages(&id, messages, tokens).await
}

#[tauri::command]
pub async fn create_agent_session(
    name: String,
    model: String,
    provider: Option<String>,
    project_id: Option<String>,
    reasoning_mode: Option<String>,
    supports_thinking: Option<bool>,
) -> Result<AgentSession, String> {
    let provider = provider.unwrap_or_else(|| "ollama".to_string());
    let requested_project_id = project_id.clone();
    let mut session =
        session_store::create_full(&name, &model, &provider, false, project_id).await?;
    if let Some(pid) = requested_project_id.as_deref() {
        if let Some(path) = super::agent_working_dir::project_path_for_id(pid).await {
            if session_store::update_working_dir(&session.id, &path)
                .await
                .is_ok()
            {
                if let Ok(updated) = session_store::get(&session.id).await {
                    session = updated;
                }
            }
        }
    }
    if reasoning_mode.is_some() {
        session_store::update_reasoning(&session.id, reasoning_mode, supports_thinking).await?;
        if let Ok(updated) = session_store::get(&session.id).await {
            session = updated;
        }
    }
    Ok(session)
}

#[tauri::command]
pub async fn rename_agent_session(id: String, name: String) -> Result<(), String> {
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
    session_store::update_model(&id, &model, &provider, reasoning_mode, supports_thinking).await
}

#[tauri::command]
pub async fn update_session_reasoning(
    id: String,
    reasoning_mode: Option<String>,
    supports_thinking: Option<bool>,
) -> Result<(), String> {
    session_store::update_reasoning(&id, reasoning_mode, supports_thinking).await
}

#[tauri::command]
pub async fn set_session_plan_mode(id: String, enabled: bool) -> Result<(), String> {
    crate::services::agent_local::tool_plan::set_enabled(&id, enabled).await
}

#[tauri::command]
pub async fn delete_agent_session(id: String) -> Result<(), String> {
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
pub async fn export_agent_session_markdown(id: String) -> Result<String, String> {
    session_store::export_markdown(&id).await
}

#[tauri::command]
pub async fn truncate_and_replace_at(
    session_id: String,
    message_id: String,
    replacement: Option<AgentMessage>,
) -> Result<(), String> {
    session_store::truncate_and_replace(&session_id, &message_id, replacement).await
}
