use super::session_security;
use super::types_session::AgentSessionMeta;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IndexFingerprint {
    pub len: u64,
    pub modified: Option<SystemTime>,
}

pub(super) fn index_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join("agent-sessions")
        .join("index.json")
}

pub(super) async fn index_fingerprint(path: &Path) -> Option<IndexFingerprint> {
    let metadata = tokio::fs::metadata(path).await.ok()?;
    Some(IndexFingerprint {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

pub(super) async fn read_index_raw() -> Vec<AgentSessionMeta> {
    let path = index_path();
    if let Ok(data) = tokio::fs::read_to_string(&path).await {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub(super) async fn write_index(entries: &[AgentSessionMeta]) -> Result<(), String> {
    let dir = crate::services::paths::data_dir().join("agent-sessions");
    write_index_to(&dir, entries).await
}

pub(crate) async fn write_index_to(
    dir: &Path,
    entries: &[AgentSessionMeta],
) -> Result<(), String> {
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|_| "index indisponible".to_string())?;
    let path = dir.join("index.json");
    let mut value = serde_json::to_value(entries).map_err(|_| "index invalide".to_string())?;
    session_security::sanitize_session_value(&mut value);
    let data = serde_json::to_vec_pretty(&value).map_err(|_| "index invalide".to_string())?;
    crate::services::private_store::atomic_write_async(path, data).await
}
