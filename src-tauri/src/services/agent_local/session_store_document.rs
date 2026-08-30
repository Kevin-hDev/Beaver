use super::session_limits::MAX_SESSION_FILE_BYTES;
use super::session_store::validate_session_id;
use super::types_session::AgentSession;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SessionReadError {
    Unavailable,
    Invalid,
}

pub(crate) struct PreparedSessionDocument {
    #[cfg(test)]
    session: AgentSession,
    data: Vec<u8>,
}

impl PreparedSessionDocument {
    #[cfg(test)]
    pub(super) fn session(&self) -> &AgentSession {
        &self.session
    }
}

impl SessionReadError {
    pub(super) const fn message(self) -> &'static str {
        match self {
            Self::Unavailable => "Session indisponible",
            Self::Invalid => "Session invalide",
        }
    }
}

pub(super) async fn read_from_dir(dir: &Path, id: &str) -> Result<AgentSession, SessionReadError> {
    let path = path_in(dir, id).map_err(|_| SessionReadError::Unavailable)?;
    read_from_path(path).await
}

pub(super) async fn read_from_path(path: PathBuf) -> Result<AgentSession, SessionReadError> {
    let data = match crate::services::private_store::read_bounded_regular_async(
        path.clone(),
        MAX_SESSION_FILE_BYTES,
    )
    .await
    .map_err(|_| SessionReadError::Unavailable)?
    {
        crate::services::private_store::BoundedFile::Missing => {
            return Err(SessionReadError::Unavailable);
        }
        crate::services::private_store::BoundedFile::Content(data) => data,
    };
    let loaded =
        super::session_migration::read(&data, path).map_err(|_| SessionReadError::Invalid)?;
    super::session_migration::acknowledge_current(&loaded)
        .await
        .map_err(|_| SessionReadError::Unavailable)?;
    Ok(loaded.into_session())
}

pub(super) async fn write_to_dir(dir: &Path, session: &AgentSession) -> Result<(), String> {
    write_to_path(path_in(dir, &session.id)?, session).await
}

pub(super) async fn write_to_path(path: PathBuf, session: &AgentSession) -> Result<(), String> {
    let prepared = prepare(session).await?;
    write_prepared_to_path(path, prepared).await
}

pub(super) async fn prepare(session: &AgentSession) -> Result<PreparedSessionDocument, String> {
    super::session_migration_wire::validate_current_writable(session)
        .map_err(|_| super::session_limits::save_failed())?;
    let mut value = serde_json::to_value(session)
        .map_err(|_| "Sauvegarde de session impossible".to_string())?;
    super::session_permission_state::merge_into_serialized(&session.id, &mut value).await;
    super::session_security::sanitize_session_value(&mut value);
    super::session_store_compaction::compact_tool_history(&mut value);
    let data = serde_json::to_vec_pretty(&value)
        .map_err(|_| "Sauvegarde de session impossible".to_string())?;
    super::session_limits::validate_serialized_size(data.len())?;
    let parsed = super::session_migration_wire::parse_v3(&data)
        .map_err(|_| super::session_limits::save_failed())?;
    super::session_migration_wire::validate_current_writable(&parsed)
        .map_err(|_| super::session_limits::save_failed())?;
    #[cfg(test)]
    let document = PreparedSessionDocument {
        session: parsed,
        data,
    };
    #[cfg(not(test))]
    let document = {
        drop(parsed);
        PreparedSessionDocument { data }
    };
    Ok(document)
}

pub(super) async fn write_prepared_to_path(
    path: PathBuf,
    prepared: PreparedSessionDocument,
) -> Result<(), String> {
    let data = prepared.data;
    match crate::services::private_store::read_bounded_regular_async(
        path.clone(),
        MAX_SESSION_FILE_BYTES,
    )
    .await
    .map_err(|_| super::session_limits::save_failed())?
    {
        crate::services::private_store::BoundedFile::Missing => {
            crate::services::private_store::atomic_write_async(path, data)
                .await
                .map_err(|_| super::session_limits::save_failed())
        }
        crate::services::private_store::BoundedFile::Content(current) => {
            let loaded = match super::session_migration::read(&current, path.clone()) {
                Ok(loaded) => loaded,
                Err(_) => {
                    let backup = super::session_migration_backup::corrupt_backup_path(&path)?;
                    super::session_migration_backup::ensure_exact_backup(&backup, &current).await?;
                    return crate::services::private_store::atomic_write_async(path, data)
                        .await
                        .map_err(|_| super::session_limits::save_failed());
                }
            };
            match loaded.version() {
                super::session_migration::LoadedVersion::V1
                | super::session_migration::LoadedVersion::V2 => {
                    super::session_migration::commit_migrated_bytes(&loaded, data).await
                }
                super::session_migration::LoadedVersion::V3 => {
                    crate::services::private_store::atomic_write_async(path, data)
                        .await
                        .map_err(|_| super::session_limits::save_failed())
                }
                super::session_migration::LoadedVersion::Future(_) => {
                    Err(super::session_limits::save_failed())
                }
            }
        }
    }
}

fn path_in(dir: &Path, id: &str) -> Result<PathBuf, String> {
    validate_session_id(id)?;
    Ok(dir.join(format!("{id}.json")))
}

#[cfg(test)]
mod tests {
    use super::super::session_limits::MAX_SESSION_FILE_BYTES;
    use super::{prepare, read_from_path, write_prepared_to_path};

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

    #[tokio::test]
    async fn corrupt_session_gets_a_distinct_backup_when_a_v1_backup_exists() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("session.json");
        let legacy_backup = root.path().join("session.json.v1.bak");
        let corrupt = b"{ definitely-not-json";
        crate::services::private_store::atomic_write(&path, corrupt).unwrap();
        crate::services::private_store::atomic_write(&legacy_backup, b"legacy-v1").unwrap();
        let session = super::super::session_migration_wire::parse_v1(include_bytes!(
            "../../../test-fixtures/agent-session-v1-synthetic.json"
        ))
        .unwrap();
        let prepared = prepare(&session).await.unwrap();

        write_prepared_to_path(path, prepared).await.unwrap();

        assert_eq!(std::fs::read(legacy_backup).unwrap(), b"legacy-v1");
        let corrupt_backups = std::fs::read_dir(root.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".corrupt."))
            .collect::<Vec<_>>();
        assert_eq!(corrupt_backups.len(), 1);
        assert_eq!(std::fs::read(corrupt_backups[0].path()).unwrap(), corrupt);
    }
}
