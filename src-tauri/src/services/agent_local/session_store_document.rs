use super::session_store::validate_session_id;
use super::types_session::AgentSession;
use std::path::{Path, PathBuf};

const MAX_SESSION_FILE_BYTES: u64 = 32 * 1024 * 1024;

pub(super) async fn read_from_dir(dir: &Path, id: &str) -> Result<AgentSession, String> {
    read_from_path(path_in(dir, id)?).await
}

pub(super) async fn read_from_path(path: PathBuf) -> Result<AgentSession, String> {
    let data = match crate::services::private_store::read_bounded_regular_async(
        path,
        MAX_SESSION_FILE_BYTES,
    )
    .await
    .map_err(|_| "Session indisponible".to_string())?
    {
        crate::services::private_store::BoundedFile::Missing => {
            return Err("Session indisponible".to_string());
        }
        crate::services::private_store::BoundedFile::Content(data) => data,
    };
    serde_json::from_slice(&data).map_err(|_| "Session invalide".to_string())
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

#[cfg(test)]
mod tests {
    use super::{read_from_path, MAX_SESSION_FILE_BYTES};

    #[tokio::test]
    async fn rejects_an_oversized_session_before_allocating_it() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("large.json");
        std::fs::File::create(&path)
            .unwrap()
            .set_len(MAX_SESSION_FILE_BYTES + 1)
            .unwrap();

        assert!(read_from_path(path).await.is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn rejects_a_symbolic_session_document() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target.json");
        let path = root.path().join("session.json");
        std::fs::write(&target, b"{}").unwrap();
        symlink(target, &path).unwrap();

        assert!(read_from_path(path).await.is_err());
    }
}
