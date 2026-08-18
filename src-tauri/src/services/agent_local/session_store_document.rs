use super::session_store::validate_session_id;
use super::types_session::AgentSession;
use std::path::{Path, PathBuf};

pub(super) async fn read_from_dir(dir: &Path, id: &str) -> Result<AgentSession, String> {
    let data = tokio::fs::read_to_string(path_in(dir, id)?)
        .await
        .map_err(|_| "Session indisponible".to_string())?;
    serde_json::from_str(&data).map_err(|_| "Session invalide".to_string())
}

pub(super) async fn write_to_dir(dir: &Path, session: &AgentSession) -> Result<(), String> {
    write_to_path(path_in(dir, &session.id)?, session).await
}

pub(super) async fn write_to_path(path: PathBuf, session: &AgentSession) -> Result<(), String> {
    let mut value = serde_json::to_value(session)
        .map_err(|_| "Sauvegarde de session impossible".to_string())?;
    super::session_permission_state::merge_into_serialized(&session.id, &mut value).await;
    super::session_security::sanitize_session_value(&mut value);
    super::session_store_compaction::compact_tool_history(&mut value);
    let data = serde_json::to_string_pretty(&value)
        .map_err(|_| "Sauvegarde de session impossible".to_string())?;
    crate::services::private_store::atomic_write_async(path, data.into_bytes())
        .await
        .map_err(|_| "Sauvegarde de session impossible".to_string())
}

fn path_in(dir: &Path, id: &str) -> Result<PathBuf, String> {
    validate_session_id(id)?;
    Ok(dir.join(format!("{id}.json")))
}
