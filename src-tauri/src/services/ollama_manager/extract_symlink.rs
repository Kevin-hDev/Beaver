use super::error::OllamaErrorCode;
use super::extract_root::relative_components;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const MAX_SYMLINK_DEPTH: usize = 64;

pub(super) fn validate_deferred_symlinks(
    archive_names: &HashSet<PathBuf>,
    symlinks: &[(PathBuf, PathBuf)],
) -> Result<(), OllamaErrorCode> {
    let targets = symlinks.iter().cloned().collect::<HashMap<_, _>>();
    for name in targets.keys() {
        validate_chain(name, archive_names, &targets)?;
    }
    Ok(())
}

fn validate_chain(
    first: &Path,
    archive_names: &HashSet<PathBuf>,
    targets: &HashMap<PathBuf, PathBuf>,
) -> Result<(), OllamaErrorCode> {
    let mut current = first.to_path_buf();
    let mut visited = HashSet::new();
    for _ in 0..MAX_SYMLINK_DEPTH {
        if !visited.insert(current.clone()) {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let Some(target) = targets.get(&current) else {
            return Ok(());
        };
        current = resolve_target(&current, target)?;
        if !archive_names.contains(&current) {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
    }
    Err(OllamaErrorCode::OllamaBundleInvalid)
}

fn resolve_target(name: &Path, target: &Path) -> Result<PathBuf, OllamaErrorCode> {
    let resolved = name.parent().unwrap_or_else(|| Path::new("")).join(target);
    Ok(relative_components(&resolved)?.into_iter().collect())
}
