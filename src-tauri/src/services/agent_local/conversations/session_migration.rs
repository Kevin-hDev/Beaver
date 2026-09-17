#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

use zeroize::Zeroizing;

use super::session_limits::{self, CURRENT_SESSION_SCHEMA_VERSION};
pub use super::session_migration_version::SessionVersion as LoadedVersion;
use super::types_session::AgentSession;

pub struct LoadedSession {
    session: AgentSession,
    path: PathBuf,
    version: LoadedVersion,
    original: Option<Zeroizing<Vec<u8>>>,
}

impl LoadedSession {
    #[cfg(test)]
    pub fn session(&self) -> &AgentSession {
        &self.session
    }

    pub fn into_session(self) -> AgentSession {
        self.session
    }

    pub const fn version(&self) -> LoadedVersion {
        self.version
    }
}

pub fn read(bytes: &[u8], path: PathBuf) -> Result<LoadedSession, String> {
    session_limits::validate_serialized_size(bytes.len())
        .map_err(|_| session_limits::invalid_session())?;
    let version = super::session_migration_version::version(bytes)?;
    let mut session = match version {
        LoadedVersion::Legacy(1) => super::session_migration_wire::parse_v1(bytes)?,
        LoadedVersion::Legacy(2) => super::session_migration_wire::parse_v2(bytes)?,
        LoadedVersion::Legacy(3) => super::session_migration_wire::parse_v3(bytes)?,
        LoadedVersion::Legacy(4) => super::session_migration_wire::parse_v4(bytes)?,
        LoadedVersion::Legacy(5) => super::session_migration_wire::parse_v5(bytes)?,
        LoadedVersion::Legacy(6) => super::session_migration_wire::parse_v6(bytes)?,
        LoadedVersion::Current => super::session_migration_wire::parse_v7(bytes)?,
        LoadedVersion::Future(value) => super::session_migration_wire::parse_future(bytes, value)?,
        LoadedVersion::Legacy(_) => return Err(session_limits::invalid_session()),
    };
    super::stream_diagnostics_history::normalize(&mut session);
    Ok(LoadedSession {
        session,
        path,
        version,
        original: version
            .legacy_number()
            .map(|_| Zeroizing::new(bytes.to_vec())),
    })
}

pub(super) async fn commit_migrated_bytes(
    loaded: &LoadedSession,
    bytes: Vec<u8>,
) -> Result<(), String> {
    let backup = migration_backup_path(loaded)?;
    session_limits::validate_serialized_size(bytes.len())?;
    let original = loaded
        .original
        .as_deref()
        .ok_or_else(session_limits::save_failed)?;
    super::session_migration_backup::publish(&loaded.path, backup, original, bytes).await
}

#[cfg(test)]
pub(super) async fn commit_current_fail_before_rename(
    loaded: &LoadedSession,
) -> Result<(), String> {
    let backup = migration_backup_path(loaded)?;
    let original = loaded
        .original
        .as_deref()
        .ok_or_else(session_limits::save_failed)?;
    super::session_migration_backup::ensure_exact_backup(&backup, original).await?;
    let path = loaded.path.clone();
    let bytes = serialize_current(loaded.session())?;
    tokio::task::spawn_blocking(move || {
        crate::services::private_store::atomic_write_fail_before_replace(&path, &bytes)
    })
    .await
    .map_err(|_| session_limits::save_failed())?
}

pub(super) async fn acknowledge_current(loaded: &LoadedSession) -> Result<(), String> {
    if loaded.version == LoadedVersion::Current {
        for version in 1..CURRENT_SESSION_SCHEMA_VERSION {
            let backup =
                super::session_migration_backup::versioned_backup_path(&loaded.path, version)?;
            if super::session_migration_backup::acknowledge_path(
                backup,
                !loaded.session.messages.is_empty(),
            )
            .await
            .is_err()
            {
                log::warn!("session_migration_backup_cleanup_failed");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn backup_path(path: &Path, version: u16) -> Result<PathBuf, String> {
    super::session_migration_backup::versioned_backup_path(path, version)
}

fn migration_backup_path(loaded: &LoadedSession) -> Result<PathBuf, String> {
    let version = loaded
        .version
        .legacy_number()
        .ok_or_else(session_limits::save_failed)?;
    super::session_migration_backup::versioned_backup_path(&loaded.path, version)
}

#[cfg(test)]
pub(super) fn serialize_current(session: &AgentSession) -> Result<Vec<u8>, String> {
    if session.schema_version != CURRENT_SESSION_SCHEMA_VERSION {
        return Err(session_limits::save_failed());
    }
    super::session_migration_wire::validate_current_writable(session)
        .map_err(|_| session_limits::save_failed())?;
    let bytes = serde_json::to_vec_pretty(session).map_err(|_| session_limits::save_failed())?;
    session_limits::validate_serialized_size(bytes.len())?;
    Ok(bytes)
}

#[cfg(test)]
pub fn is_legacy_local_id(value: &str) -> bool {
    super::session_migration_ids::is_legacy_local_id(value)
}
