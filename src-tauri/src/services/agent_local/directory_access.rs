use serde::Serialize;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

const MAX_ALLOWED_PATHS: usize = 32;
const MAX_PATH_CHARS: usize = 4_096;
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
        let canonical = Path::new(value.trim())
            .canonicalize()
            .map_err(|_| ACCESS_ERROR.to_string())?;
        if !canonical.is_dir() {
            return Err(ACCESS_ERROR.to_string());
        }
        let text = canonical
            .to_str()
            .ok_or_else(|| ACCESS_ERROR.to_string())?
            .to_string();
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
    let paths = crate::services::config::read_config()
        .map_err(|_| ACCESS_ERROR.to_string())?
        .advanced
        .allowed_paths;
    normalize_allowed_paths(paths).map(|paths| paths.into_iter().map(PathBuf::from).collect())
}

pub fn decision(path: &Path) -> Result<DirectoryAccessDecision, String> {
    let candidate = canonical_access_path(path)?;
    let roots = configured_roots()?;
    Ok(DirectoryAccessDecision {
        allowed: is_path_in_roots(&candidate, &roots),
        allowed_paths: roots.iter().filter_map(|root| path_text(root)).collect(),
    })
}

pub fn ensure_allowed(path: &Path) -> Result<PathBuf, String> {
    let candidate = canonical_access_path(path)?;
    let roots = configured_roots()?;
    if is_path_in_roots(&candidate, &roots) {
        Ok(candidate)
    } else {
        Err(ACCESS_ERROR.to_string())
    }
}

pub fn shell_access_allowed() -> bool {
    configured_roots()
        .map(|roots| roots_allow_shell(&roots))
        .unwrap_or(false)
}

pub(crate) fn roots_allow_shell(roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| root.parent().is_none())
}

pub async fn project_path(project_id: &str) -> Result<PathBuf, String> {
    let project = super::project_store::list()
        .await
        .map_err(|_| ACCESS_ERROR.to_string())?
        .into_iter()
        .find(|project| project.id == project_id)
        .ok_or_else(|| ACCESS_ERROR.to_string())?;
    ensure_allowed(Path::new(&project.path))
}

pub async fn ensure_session_allowed(
    session: &super::types_session::AgentSession,
) -> Result<(), String> {
    if let Some(project_id) = session.project_id.as_deref() {
        project_path(project_id).await?;
    } else if !session.working_dir_managed && !session.working_dir.trim().is_empty() {
        ensure_allowed(Path::new(&session.working_dir))?;
    }
    Ok(())
}

pub(crate) fn is_path_in_roots(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

fn canonical_access_path(path: &Path) -> Result<PathBuf, String> {
    validate_shape(path.to_str().ok_or_else(|| ACCESS_ERROR.to_string())?)?;
    if path.exists() {
        return path.canonicalize().map_err(|_| ACCESS_ERROR.to_string());
    }
    let mut existing = path;
    let mut suffix = Vec::new();
    while !existing.exists() {
        let name = existing
            .file_name()
            .ok_or_else(|| ACCESS_ERROR.to_string())?;
        suffix.push(name.to_os_string());
        existing = existing
            .parent()
            .ok_or_else(|| ACCESS_ERROR.to_string())?;
    }
    let mut canonical = existing
        .canonicalize()
        .map_err(|_| ACCESS_ERROR.to_string())?;
    for part in suffix.into_iter().rev() {
        canonical.push(part);
    }
    Ok(canonical)
}

fn validate_shape(value: &str) -> Result<(), String> {
    let value = value.trim();
    let path = Path::new(value);
    if value.is_empty()
        || value.chars().count() > MAX_PATH_CHARS
        || value.chars().any(char::is_control)
        || !path.is_absolute()
        || path.components().any(|part| matches!(part, Component::ParentDir))
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
