use super::{root, HashSet, Path, PathBuf, MAX_ARTIFACTS_PER_EXTENSION};

pub(super) fn cleanup_entry(
    path: &Path,
    referenced: &HashSet<PathBuf>,
    active: &HashSet<PathBuf>,
) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if metadata.is_dir() && name.starts_with(".staging-") {
        if active.contains(path) {
            return Ok(());
        }
        return std::fs::remove_dir_all(path).map_err(|_| invalid());
    }
    if !metadata.is_dir() || super::super::validation::identifier(name).is_err() {
        return Ok(());
    }
    for (index, child) in std::fs::read_dir(path).map_err(|_| invalid())?.enumerate() {
        if index >= MAX_ARTIFACTS_PER_EXTENSION {
            return Err(invalid());
        }
        let child = child.map_err(|_| invalid())?.path();
        let child_name = child
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if valid_token(child_name, 64) && !referenced.contains(&child) {
            std::fs::remove_dir_all(&child).map_err(|_| invalid())?;
        }
    }
    remove_empty_parent(path)
}

pub(super) fn remove_empty_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent != root()
        && parent.exists()
        && std::fs::read_dir(parent)
            .map_err(|_| invalid())?
            .next()
            .is_none()
    {
        std::fs::remove_dir(parent).map_err(|_| invalid())?;
    }
    Ok(())
}

pub(super) fn valid_token(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn invalid() -> String {
    super::super::ui_contract::UI_DIAGNOSTIC_UI_ARTIFACT_INVALID.to_string()
}
// Called only by the still-owned staging after its entire producer scope stopped.
pub(super) fn reset_for_retry(
    output: &std::path::Path,
    temporary: &std::path::Path,
) -> Result<(), String> {
    for path in [output, temporary] {
        let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(invalid());
        }
        std::fs::remove_dir_all(path).map_err(|_| invalid())?;
        crate::services::private_store::ensure_private_dir(path).map_err(|_| invalid())?;
    }
    Ok(())
}
