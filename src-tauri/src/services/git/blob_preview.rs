use super::history::{find_reachable_commit, open_current_branch};
use std::path::{Component, Path};

pub const MAX_GIT_BLOB_SIZE: usize = 50 * 1024 * 1024;
pub const MAX_GIT_BINARY_PREVIEW_SIZE: usize = 10 * 1024 * 1024;
const MAX_PATH_LEN: usize = 4096;

pub fn read_blob_with_limit(
    repo_path: &Path,
    expected_branch: &str,
    commit_id: &str,
    file_path: &str,
    use_parent: bool,
    max_size: usize,
) -> Result<Vec<u8>, String> {
    validate_repo_path(file_path)?;
    let (repo, head) = open_current_branch(repo_path, expected_branch)?;
    let commit = find_reachable_commit(&repo, head, commit_id)?;
    let tree = if use_parent {
        commit
            .parent(0)
            .and_then(|parent| parent.tree())
            .map_err(|_| unavailable())?
    } else {
        commit.tree().map_err(|_| unavailable())?
    };
    let entry = tree
        .get_path(Path::new(file_path))
        .map_err(|_| unavailable())?;
    let object = entry.to_object(&repo).map_err(|_| unavailable())?;
    let blob = object.as_blob().ok_or_else(unavailable)?;
    if blob.content().len() > max_size.min(MAX_GIT_BLOB_SIZE) {
        return Err(unavailable());
    }
    Ok(blob.content().to_vec())
}

pub(super) fn validate_repo_path(value: &str) -> Result<(), String> {
    let path = Path::new(value);
    if value.is_empty()
        || value.len() > MAX_PATH_LEN
        || value.contains('\0')
        || path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(unavailable());
    }
    Ok(())
}

fn unavailable() -> String {
    "Aperçu Git indisponible".to_string()
}
