use crate::services::agent_local::types_session::{AgentSession, AgentSessionMeta};
use chrono::Utc;
use uuid::Uuid;

pub use super::session_id::validate_session_id;
pub(crate) use super::session_locks::lock_session;
pub use super::session_locks::remove_session_lock;
pub use super::session_store_messages::add_messages;

pub async fn create_with_flags(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
) -> Result<AgentSession, String> {
    create_full(name, model, provider, is_heartbeat, None).await
}

pub async fn create_gateway(
    name: &str,
    model: &str,
    provider: &str,
    gateway_channel_key: String,
) -> Result<AgentSession, String> {
    let mut session = create_full(name, model, provider, false, None).await?;
    session.is_gateway = true;
    session.gateway_channel_key = Some(gateway_channel_key);
    save(&session).await?;
    Ok(session)
}

pub async fn create_full(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
    project_id: Option<String>,
) -> Result<AgentSession, String> {
    let reasoning_mode = crate::services::reasoning::default_mode(provider, model);
    let now = Utc::now();
    let session = AgentSession {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        created_at: now,
        updated_at: Some(now),
        archived_at: None,
        model: model.to_string(),
        provider: provider.to_string(),
        thinking_enabled: crate::services::reasoning::enabled(reasoning_mode.as_deref(), false),
        reasoning_mode,
        accumulated_tokens: 0,
        messages: Vec::new(),
        todos: Vec::new(),
        todo_neglect_count: 0,
        todo_runs: Vec::new(),
        active_todo_run_id: None,
        stream_failures: Vec::new(),
        diagnostic_runs: Vec::new(),
        plan_mode_enabled: false,
        plan_runs: Vec::new(),
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        plan_approval_decision: None,
        is_heartbeat,
        is_gateway: false,
        gateway_channel_key: None,
        project_id,
        working_dir: String::new(),
        parent_session_id: None,
        subagent_type: None,
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: None,
        subagent_run_id: None,
        subagent_description: None,
        subagent_color_key: None,
        subagent_summary: None,
        subagent_last_activity: None,
        subagent_queued_prompts: Vec::new(),
        subagent_hidden_reports: Vec::new(),
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: Vec::new(),
        clone_modified_files: Vec::new(),
        clone_root_session_id: None,
        git_branch: None,
    };
    save(&session).await?;
    Ok(session)
}

/// Cherche la conversation heartbeat existante pour un couple (provider, model).
/// Retourne la plus récente si plusieurs existent, `None` sinon.
pub async fn find_heartbeat_session(provider: &str, model: &str) -> Result<Option<String>, String> {
    let metas = crate::services::agent_local::session_index::read_index().await?;
    let best = metas
        .iter()
        .filter(|m| {
            super::session_archive::is_active(m)
                && m.is_heartbeat
                && m.provider == provider
                && m.model == model
        })
        .max_by_key(|m| super::session_archive::activity_at(m))
        .map(|m| m.id.clone());
    Ok(best)
}

pub async fn get(id: &str) -> Result<AgentSession, String> {
    validate_session_id(id)?;
    let path = crate::services::paths::data_file_for_read(
        "agent-sessions",
        &format!("{id}.json"),
    )
    .await
    .map_err(|_| "Session introuvable".to_string())?;
    let data = tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| "Session introuvable".to_string())?;
    serde_json::from_str(&data).map_err(|_| "Session invalide".to_string())
}

pub async fn list() -> Result<Vec<AgentSessionMeta>, String> {
    let mut metas = crate::services::agent_local::session_index::read_index().await?;
    metas.retain(super::session_archive::is_active);
    super::session_archive::sort_recent_first(&mut metas);
    Ok(metas)
}

pub async fn save(session: &AgentSession) -> Result<(), String> {
    validate_session_id(&session.id)?;
    let path = crate::services::paths::data_file_for_write(
        "agent-sessions",
        &format!("{}.json", session.id),
    )
    .await
    .map_err(|_| "Sauvegarde de session impossible".to_string())?;
    let mut value = serde_json::to_value(session).map_err(|e| e.to_string())?;
    super::session_permission_state::merge_into_serialized(&session.id, &mut value).await;
    super::session_security::sanitize_session_value(&mut value);
    let data = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    crate::services::private_store::atomic_write_async(path, data.into_bytes()).await?;
    let meta = crate::services::agent_local::session_index::meta_from_session(session);
    let _ = crate::services::agent_local::session_index::upsert_entry(meta).await;
    Ok(())
}

pub async fn rename(id: &str, name: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let mut session = get(id).await?;
    session.name = name.to_string();
    save(&session).await
}

pub(crate) async fn delete_one(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let path = crate::services::paths::data_file_for_read(
        "agent-sessions",
        &format!("{id}.json"),
    )
    .await
    .map_err(|_| "Session introuvable".to_string())?;
    tokio::fs::remove_file(&path)
        .await
        .map_err(|_| "Suppression de session impossible".to_string())?;
    let _ = crate::services::agent_local::session_index::remove_entry(id).await;
    let _ = super::subagent_change_store::remove(id).await;
    super::session_permission_state::remove(id).await;
    // Nettoie aussi le WriteGuard persistant de la session.
    crate::services::agent_local::write_guard_registry::remove(id);
    Ok(())
}

pub async fn delete(id: &str) -> Result<(), String> {
    super::session_family::delete_family(id).await
}

pub async fn archive(id: &str) -> Result<(), String> {
    super::session_family::archive_family(id).await
}

pub async fn restore(id: &str) -> Result<(), String> {
    super::session_family::restore_with_parent(id).await
}

pub use super::session_archive::list_archived;
pub use super::session_ops::{clear_project_id, export_markdown, truncate_and_replace};
pub use super::session_store_updates::{
    switch_working_dir_to_project, update_model, update_reasoning, update_working_dir,
};

#[path = "session_store_tests.rs"]
#[cfg(test)]
mod tests;
