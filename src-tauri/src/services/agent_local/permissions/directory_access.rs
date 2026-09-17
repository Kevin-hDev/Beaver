use serde::Serialize;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

pub(crate) use super::directory_access_scope::{roots_allow_full_disk, workspace_roots};

pub(crate) const MAX_ALLOWED_PATHS: usize = 70;
pub(crate) const MAX_WORKSPACE_ROOTS: usize = MAX_ALLOWED_PATHS + 3;
pub(crate) const MAX_PATH_CHARS: usize = 4_096;
const ACCESS_ERROR: &str = "Accès au dossier refusé par les réglages.";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DirectoryAccessDecision {
    pub allowed: bool,
    pub allowed_paths: Vec<String>,
}

pub fn normalize_allowed_paths(paths: Vec<String>) -> Result<Vec<String>, String> {
    if paths.is_empty() || paths.len() > MAX_ALLOWED_PATHS {
        return Err(ACCESS_ERROR.to_string());
    }
    let mut seen = HashSet::with_capacity(paths.len());
    let mut normalized = Vec::with_capacity(paths.len());
    for value in paths {
        validate_shape(&value)?;
        let canonical = canonical_existing_directory(Path::new(value.trim()))?;
        let text = path_text(&canonical).ok_or_else(|| ACCESS_ERROR.to_string())?;
        if seen.insert(text.clone()) {
            normalized.push(text);
        }
    }
    if normalized.is_empty() {
        return Err(ACCESS_ERROR.to_string());
    }
    Ok(normalized)
}

pub fn configured_roots() -> Result<Vec<PathBuf>, String> {
    super::directory_policy::roots().map_err(|_| ACCESS_ERROR.to_string())
}

pub fn initialize_policy() -> Result<(), String> {
    super::directory_policy::initialize().map_err(|_| ACCESS_ERROR.to_string())
}

pub(crate) fn replace_policy(paths: Vec<String>) -> Result<Vec<PathBuf>, String> {
    super::directory_policy::replace(paths).map_err(|_| ACCESS_ERROR.to_string())
}

pub(crate) fn apply_cached_policy(config: &mut crate::models::ClgoConfig) {
    if let Some(paths) = super::directory_policy::cached_paths() {
        config.advanced.allowed_paths = paths;
    }
}

pub fn decision(path: &Path) -> Result<DirectoryAccessDecision, String> {
    let roots = configured_roots()?;
    decision_in_roots(path, &roots)
}

pub(crate) fn decision_in_roots(
    path: &Path,
    roots: &[PathBuf],
) -> Result<DirectoryAccessDecision, String> {
    let candidate = canonical_access_path(path)?;
    Ok(DirectoryAccessDecision {
        allowed: is_path_in_roots(&candidate, roots),
        allowed_paths: roots.iter().filter_map(|root| path_text(root)).collect(),
    })
}

pub fn ensure_allowed(path: &Path) -> Result<PathBuf, String> {
    let roots = configured_roots()?;
    ensure_allowed_in_roots(path, &roots)
}

pub(crate) fn ensure_allowed_in_roots(path: &Path, roots: &[PathBuf]) -> Result<PathBuf, String> {
    let candidate = canonical_access_path(path)?;
    if is_path_in_roots(&candidate, roots) {
        Ok(candidate)
    } else {
        Err(ACCESS_ERROR.to_string())
    }
}

pub async fn project_path(project_id: &str) -> Result<PathBuf, String> {
    let project = super::project_store::find(project_id)
        .await
        .map_err(|_| ACCESS_ERROR.to_string())?
        .ok_or_else(|| ACCESS_ERROR.to_string())?;
    ensure_allowed(Path::new(&project.path))
}

pub(crate) fn is_path_in_roots(path: &Path, roots: &[PathBuf]) -> bool {
    super::directory_access_scope::roots_allow_full_disk(roots)
        || roots.iter().any(|root| path.starts_with(root))
}

pub(crate) fn canonical_access_path(path: &Path) -> Result<PathBuf, String> {
    validate_shape(path.to_str().ok_or_else(|| ACCESS_ERROR.to_string())?)?;
    if path.exists() {
        return dunce::canonicalize(path).map_err(|_| ACCESS_ERROR.to_string());
    }
    let mut existing = path;
    let mut suffix = Vec::new();
    while !existing.exists() {
        let name = existing
            .file_name()
            .ok_or_else(|| ACCESS_ERROR.to_string())?;
        suffix.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| ACCESS_ERROR.to_string())?;
    }
    let mut canonical = dunce::canonicalize(existing).map_err(|_| ACCESS_ERROR.to_string())?;
    for part in suffix.into_iter().rev() {
        canonical.push(part);
    }
    Ok(canonical)
}

pub(crate) fn configured_roots_from_paths(paths: Vec<String>) -> Result<Vec<PathBuf>, String> {
    canonical_roots_from_paths(paths, MAX_ALLOWED_PATHS)
}

pub(crate) fn transported_roots_from_paths(paths: Vec<String>) -> Result<Vec<PathBuf>, String> {
    canonical_roots_from_paths(paths, MAX_WORKSPACE_ROOTS)
}

fn canonical_roots_from_paths(paths: Vec<String>, limit: usize) -> Result<Vec<PathBuf>, String> {
    if paths.is_empty() || paths.len() > limit {
        return Err(ACCESS_ERROR.to_string());
    }
    let mut seen = HashSet::with_capacity(paths.len());
    let roots = paths
        .into_iter()
        .filter_map(|value| {
            validate_shape(&value).ok()?;
            let path = canonical_existing_directory(Path::new(value.trim())).ok()?;
            seen.insert(path.clone()).then_some(path)
        })
        .collect::<Vec<_>>();
    if roots.is_empty() {
        Err(ACCESS_ERROR.to_string())
    } else {
        Ok(roots)
    }
}

fn canonical_existing_directory(path: &Path) -> Result<PathBuf, String> {
    let canonical = dunce::canonicalize(path).map_err(|_| ACCESS_ERROR.to_string())?;
    canonical
        .is_dir()
        .then_some(canonical)
        .ok_or_else(|| ACCESS_ERROR.to_string())
}

fn validate_shape(value: &str) -> Result<(), String> {
    let value = value.trim();
    let path = Path::new(value);
    if value.is_empty()
        || value.chars().count() > MAX_PATH_CHARS
        || value.chars().any(char::is_control)
        || !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(ACCESS_ERROR.to_string());
    }
    Ok(())
}

fn path_text(path: &Path) -> Option<String> {
    path.to_str().map(ToString::to_string)
}

#[cfg(test)]
#[path = "directory_access_tests.rs"]
mod tests;
