use super::limits::MAX_GROUP_KEY_BYTES;
use crate::services::agent_local::project_store;
use std::path::{Path, PathBuf};

const DEFAULT_GROUP: &str = "__default__";
const SESSION_GROUP_PREFIX: &str = "session:";
const INVALID: &str = "terminal-cwd-invalid";

pub async fn resolve(group_key: &str) -> Result<PathBuf, String> {
    let home = if group_key == DEFAULT_GROUP {
        dirs::home_dir().ok_or_else(invalid)?
    } else {
        PathBuf::new()
    };
    let session_group = group_key.starts_with(SESSION_GROUP_PREFIX);
    resolve_with(group_key, &home, |key| async move {
        if session_group {
            session_directory(&key).await.map(Some)
        } else {
            project_store::find(&key)
                .await
                .map(|project| project.map(|project| project.path))
        }
    })
    .await
}

async fn session_directory(session_id: &str) -> Result<String, String> {
    let session = crate::services::agent_local::session_store::get(session_id).await?;
    if let Some(project_id) = session.project_id.as_deref() {
        match project_store::find(project_id).await {
            Ok(Some(_)) => return Err(invalid()),
            Ok(None) => {}
            Err(_) => return Err(invalid()),
        }
    }
    let path = if session.working_dir_managed || session.working_dir.trim().is_empty() {
        let workspace = crate::services::agent_local::session_workspace::ensure(&session).await?;
        crate::services::agent_local::session_store::set_managed_working_dir(
            session_id,
            workspace.work.to_string_lossy().as_ref(),
        )
        .await?;
        workspace.work
    } else {
        crate::services::agent_local::directory_access::ensure_allowed(Path::new(
            &session.working_dir,
        ))?
    };
    Ok(path.to_string_lossy().into_owned())
}

pub(super) async fn resolve_with<Find, Future>(
    group_key: &str,
    home: &Path,
    find_project: Find,
) -> Result<PathBuf, String>
where
    Find: FnOnce(String) -> Future,
    Future: std::future::Future<Output = Result<Option<String>, String>>,
{
    if !valid_group_key(group_key) {
        return Err(invalid());
    }
    let candidate = if group_key == DEFAULT_GROUP {
        home.to_path_buf()
    } else {
        let registry_key = group_key
            .strip_prefix(SESSION_GROUP_PREFIX)
            .unwrap_or(group_key);
        let path = find_project(registry_key.to_string())
            .await
            .map_err(|_| invalid())?
            .ok_or_else(invalid)?;
        PathBuf::from(path)
    };
    canonical_directory(&candidate)
}

fn valid_group_key(group_key: &str) -> bool {
    !group_key.is_empty()
        && group_key.len() <= MAX_GROUP_KEY_BYTES
        && !group_key
            .as_bytes()
            .iter()
            .any(|byte| matches!(byte, 0 | b'\r' | b'\n'))
}

fn canonical_directory(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(invalid());
    }
    let canonical = dunce::canonicalize(path).map_err(|_| invalid())?;
    canonical.is_dir().then_some(canonical).ok_or_else(invalid)
}

fn invalid() -> String {
    INVALID.to_string()
}
