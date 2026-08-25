use crate::services::agent_local::types_session::{AgentSession, AgentSessionMeta};
pub use super::session_id::validate_session_id;
pub(crate) use super::session_locks::lock_session;
pub use super::session_locks::remove_session_lock;
pub use super::session_store_messages::add_messages;
#[cfg(test)]
pub use super::session_store_messages::add_messages_with_context;
pub use super::session_store_create::{
    create_full, create_gateway, create_with_project, create_with_project_and_fast_mode,
};

pub async fn get(id: &str) -> Result<AgentSession, String> {
    validate_session_id(id)?;
    let path = crate::services::paths::data_file_for_read("agent-sessions", &format!("{id}.json"))
        .await
        .map_err(|_| "Session introuvable".to_string())?;
    super::session_store_document::read_from_path(path)
        .await
        .map_err(|error| match error {
            super::session_store_document::SessionReadError::Invalid => error.message().to_string(),
            super::session_store_document::SessionReadError::Unavailable => {
                "Session introuvable".to_string()
            }
        })
}

pub async fn list() -> Result<Vec<AgentSessionMeta>, String> {
    let mut metas = crate::services::agent_local::session_index::read_index().await?;
    metas.retain(super::session_archive::is_active);
    let ranks = super::session_order::ranks().await;
    super::session_archive::sort_for_display(&mut metas, &ranks);
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
    super::session_store_document::write_to_path(path, session).await?;
    let meta = crate::services::agent_local::session_index::meta_from_session(session);
    if crate::services::agent_local::session_index::upsert_entry(meta)
        .await
        .is_err()
        && crate::services::agent_local::session_index::repair_after_upsert_failure()
            .await
            .is_err()
    {
        // Le document est l'autorité déjà durable : invalider force la prochaine lecture
        // à reconstruire la projection sans annoncer à tort que sa sauvegarde a échoué.
        crate::services::agent_local::session_index::invalidate_reconcile_fingerprint().await;
    }
    Ok(())
}

pub(crate) async fn read_from_dir(dir: &std::path::Path, id: &str) -> Result<AgentSession, String> {
    super::session_store_document::read_from_dir(dir, id)
        .await
        .map_err(|error| error.message().to_string())
}

pub(crate) async fn write_to_dir(
    dir: &std::path::Path,
    session: &AgentSession,
) -> Result<(), String> {
    super::session_store_document::write_to_dir(dir, session).await
}

pub async fn rename(id: &str, name: &str) -> Result<(), String> {
    super::session_store_updates::update_locked(id, |session| {
        session.name = name.to_string();
    })
    .await
}

pub(crate) async fn delete_one(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let directory = crate::services::paths::data_dir().join("agent-sessions");
    tokio::task::spawn_blocking({
        let directory = directory.clone();
        let id = id.to_string();
        move || super::session_artifacts::remove_all_in(&directory, &id)
    })
    .await
    .map_err(|_| "Suppression de session impossible".to_string())??;
    let _ = crate::services::agent_local::session_index::remove_entry(id).await;
    let _ = super::subagent_change_store::remove(id).await;
    super::extension_session_state::remove(id).await;
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
pub use super::session_ops::{clear_project_id, export_markdown};
pub use super::session_store_updates::{
    refresh_working_dir, set_managed_working_dir, switch_working_dir_to_project,
    update_fast_mode, update_model, update_reasoning, update_working_dir,
};

#[path = "session_store_tests.rs"]
#[cfg(test)]
mod tests;
