use crate::services::agent_local::project_store;
use std::path::{Path, PathBuf};

const DEFAULT_GROUP: &str = "__default__";
const MAX_GROUP_KEY_BYTES: usize = 128;
const INVALID: &str = "terminal-cwd-invalid";

// The terminal spawn switches to this boundary atomically in Task 4.
#[allow(dead_code)]
pub async fn resolve(group_key: &str) -> Result<PathBuf, String> {
    let home = if group_key == DEFAULT_GROUP {
        dirs::home_dir().ok_or_else(invalid)?
    } else {
        PathBuf::new()
    };
    resolve_with(group_key, &home, |key| async move {
        project_store::find(&key)
            .await
            .map(|project| project.map(|project| project.path))
    })
    .await
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
        let path = find_project(group_key.to_string())
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
