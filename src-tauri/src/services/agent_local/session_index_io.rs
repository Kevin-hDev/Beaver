use super::session_security;
use super::types_session::AgentSessionMeta;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub(super) const MAX_INDEX_FILE_BYTES: u64 = 4 * 1024 * 1024;
pub(super) const MAX_INDEX_ENTRIES: usize = 4_096;

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
    read_index_from(&index_path()).await.unwrap_or_default()
}

pub(super) async fn read_index_from(path: &Path) -> Result<Vec<AgentSessionMeta>, String> {
    let data = match crate::services::private_store::read_bounded_regular_async(
        path.to_path_buf(),
        MAX_INDEX_FILE_BYTES,
    )
    .await?
    {
        crate::services::private_store::BoundedFile::Missing => {
            return Err("index indisponible".to_string());
        }
        crate::services::private_store::BoundedFile::Content(data) => data,
    };
    parse_index(&data)
}

pub(super) fn parse_index(data: &[u8]) -> Result<Vec<AgentSessionMeta>, String> {
    let entries: Vec<AgentSessionMeta> =
        serde_json::from_slice(data).map_err(|_| "index invalide".to_string())?;
    if entries.len() > MAX_INDEX_ENTRIES {
        return Err("index invalide".to_string());
    }
    Ok(entries)
}

pub(super) async fn write_index(entries: &[AgentSessionMeta]) -> Result<(), String> {
    let dir = crate::services::paths::data_dir().join("agent-sessions");
    write_index_to(&dir, entries).await
}

pub(crate) async fn write_index_to(
    dir: &Path,
    entries: &[AgentSessionMeta],
) -> Result<(), String> {
    if entries.len() > MAX_INDEX_ENTRIES {
        return Err("index invalide".to_string());
    }
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|_| "index indisponible".to_string())?;
    let path = dir.join("index.json");
    let mut value = serde_json::to_value(entries).map_err(|_| "index invalide".to_string())?;
    session_security::sanitize_session_value(&mut value);
    let data = serde_json::to_vec_pretty(&value).map_err(|_| "index invalide".to_string())?;
    crate::services::private_store::atomic_write_async(path, data).await
}
