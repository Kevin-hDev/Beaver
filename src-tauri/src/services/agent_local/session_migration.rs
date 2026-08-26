#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

use zeroize::Zeroizing;

use super::session_limits::{self, CURRENT_SESSION_SCHEMA_VERSION};
use super::session_migration_wire::WireVersion;
use super::types_session::AgentSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadedVersion {
    V1,
    V2,
    Future(u16),
}

pub struct LoadedSession {
    session: AgentSession,
    path: PathBuf,
    version: LoadedVersion,
    original: Option<Zeroizing<Vec<u8>>>,
}

impl LoadedSession {
    #[allow(dead_code, reason = "public migration API consumed by staged session owners")]
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
    let version = super::session_migration_wire::version(bytes)?;
    let (session, version) = match version {
        WireVersion::V1 => (
            super::session_migration_wire::parse_v1(bytes)?,
            LoadedVersion::V1,
        ),
        WireVersion::V2 => (
            super::session_migration_wire::parse_v2(bytes)?,
            LoadedVersion::V2,
        ),
        WireVersion::Future(value) => (
            super::session_migration_wire::parse_future(bytes, value)?,
            LoadedVersion::Future(value),
        ),
    };
    Ok(LoadedSession {
        session,
        path,
        version,
        original: (version == LoadedVersion::V1).then(|| Zeroizing::new(bytes.to_vec())),
    })
}

#[allow(dead_code, reason = "public migration API consumed by staged session owners")]
pub async fn commit_v2(loaded: &LoadedSession) -> Result<(), String> {
    let bytes = serialize_v2(loaded.session())?;
    commit_v2_bytes(loaded, bytes).await
}

pub(super) async fn commit_v2_bytes(
    loaded: &LoadedSession,
    bytes: Vec<u8>,
) -> Result<(), String> {
    if loaded.version != LoadedVersion::V1 {
        return Err(session_limits::save_failed());
    }
    session_limits::validate_serialized_size(bytes.len())?;
    let original = loaded
        .original
        .as_deref()
        .ok_or_else(session_limits::save_failed)?;
    super::session_migration_backup::publish(&loaded.path, original, bytes).await
}

#[cfg(test)]
pub(super) async fn commit_v2_fail_before_rename(loaded: &LoadedSession) -> Result<(), String> {
    if loaded.version != LoadedVersion::V1 {
        return Err(session_limits::save_failed());
    }
    let backup = super::session_migration_backup::backup_path(&loaded.path)?;
    let original = loaded
        .original
        .as_deref()
        .ok_or_else(session_limits::save_failed)?;
    super::session_migration_backup::ensure_exact_backup(&backup, original).await?;
    let path = loaded.path.clone();
    let bytes = serialize_v2(loaded.session())?;
    tokio::task::spawn_blocking(move || {
        crate::services::private_store::atomic_write_fail_before_replace(&path, &bytes)
    })
    .await
    .map_err(|_| session_limits::save_failed())?
}

pub(super) async fn acknowledge_v2(loaded: &LoadedSession) -> Result<(), String> {
    if loaded.version == LoadedVersion::V2
        && super::session_migration_backup::acknowledge(
            &loaded.path,
            !loaded.session.messages.is_empty(),
        )
            .await
            .is_err()
    {
        log::warn!("session_migration_backup_cleanup_failed");
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn backup_path(path: &Path) -> Result<PathBuf, String> {
    super::session_migration_backup::backup_path(path)
}

#[allow(dead_code, reason = "shared serializer for the staged public migration API")]
pub(super) fn serialize_v2(session: &AgentSession) -> Result<Vec<u8>, String> {
    if session.schema_version != CURRENT_SESSION_SCHEMA_VERSION {
        return Err(session_limits::save_failed());
    }
    super::session_migration_wire::validate_v2_writable(session)
        .map_err(|_| session_limits::save_failed())?;
    let bytes = serde_json::to_vec_pretty(session).map_err(|_| session_limits::save_failed())?;
    session_limits::validate_serialized_size(bytes.len())?;
    Ok(bytes)
}

#[cfg(test)]
pub fn is_legacy_local_id(value: &str) -> bool {
    super::session_migration_ids::is_legacy_local_id(value)
}
