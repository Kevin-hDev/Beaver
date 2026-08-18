use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;
use uuid::Uuid;

const PROJECT_STORE_UNAVAILABLE: &str =
    crate::services::private_store::error_codes::PROJECT_STORE_UNAVAILABLE;
const MAX_PROJECT_STORE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_PROJECTS: usize = 4_096;
const MAX_PROJECT_ID_BYTES: usize = 128;
const MAX_PROJECT_NAME_BYTES: usize = 512;
const MAX_PROJECT_PATH_BYTES: usize = 32_768;
static PROJECT_STORE_LOCK: Mutex<()> = Mutex::const_new(());

#[path = "project_store_recovery.rs"]
mod recovery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub order: usize,
    pub created_at: DateTime<Utc>,
}

fn projects_path() -> PathBuf {
    crate::services::paths::data_dir().join("projects.json")
}

async fn read_all() -> Result<Vec<Project>, String> {
    let _guard = PROJECT_STORE_LOCK.lock().await;
    read_all_unlocked().await
}

async fn read_all_unlocked() -> Result<Vec<Project>, String> {
    read_all_from(&projects_path()).await
}

async fn read_all_from(path: &Path) -> Result<Vec<Project>, String> {
    let data = match crate::services::private_store::read_bounded_regular_async(
        path.to_path_buf(),
        MAX_PROJECT_STORE_BYTES,
    )
    .await
    .map_err(|_| PROJECT_STORE_UNAVAILABLE.to_string())?
    {
        crate::services::private_store::BoundedFile::Missing => return Ok(Vec::new()),
        crate::services::private_store::BoundedFile::Content(data) => data,
    };
    match serde_json::from_slice::<Vec<Project>>(&data)
        .map_err(|_| PROJECT_STORE_UNAVAILABLE.to_string())
        .and_then(|projects| {
            validate_projects(&projects)?;
            Ok(projects)
        }) {
        Ok(projects) => Ok(projects),
        Err(_) => recovery::backup_and_reset(path, data).await,
    }
}

async fn write_atomic(projects: &[Project]) -> Result<(), String> {
    validate_projects(projects)?;
    let path = projects_path();
    let data = serde_json::to_string_pretty(projects)
        .map_err(|_| PROJECT_STORE_UNAVAILABLE.to_string())?;
    crate::services::private_store::atomic_write_async(path, data.into_bytes())
        .await
        .map_err(|_| PROJECT_STORE_UNAVAILABLE.to_string())
}

pub async fn list() -> Result<Vec<Project>, String> {
    let mut projects = read_all().await?;
    projects.sort_by_key(|p| p.order);
    Ok(projects)
}

pub async fn find(id: &str) -> Result<Option<Project>, String> {
    Ok(read_all()
        .await?
        .into_iter()
        .find(|project| project.id == id))
}

pub async fn add(path: &str) -> Result<Project, String> {
    let _guard = PROJECT_STORE_LOCK.lock().await;
    let canonical = canonical_existing_dir(Path::new(path))?;
    super::directory_access::ensure_allowed(&canonical)?;
    let canonical_path = canonical.to_string_lossy().to_string();
    let mut projects = read_all_unlocked().await?;
    if let Some(existing) = projects
        .iter()
        .find(|p| project_matches_canonical(&p.path, &canonical))
    {
        return Ok(existing.clone());
    }
    if projects.len() >= MAX_PROJECTS {
        return Err(PROJECT_STORE_UNAVAILABLE.to_string());
    }
    let name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Projet")
        .to_string();
    let project = Project {
        id: Uuid::new_v4().to_string(),
        name,
        path: canonical_path,
        order: projects.len(),
        created_at: Utc::now(),
    };
    projects.push(project.clone());
    write_atomic(&projects).await?;
    Ok(project)
}

pub async fn authorize_path(path: &Path) -> Result<PathBuf, String> {
    let canonical = canonical_existing_dir(path)?;
    let projects = read_all().await?;
    if projects
        .iter()
        .any(|p| path_is_inside_project(&canonical, &p.path))
    {
        return Ok(canonical);
    }
    Err("Projet non autorisé".to_string())
}

fn canonical_existing_dir(path: &Path) -> Result<PathBuf, String> {
    if path.as_os_str().is_empty() {
        return Err("Chemin de projet invalide".to_string());
    }
    let canonical = dunce::canonicalize(path).map_err(|_| "Dossier introuvable".to_string())?;
    if !canonical.is_dir() {
        return Err("Le chemin ne pointe pas vers un dossier valide".to_string());
    }
    Ok(canonical)
}

fn project_matches_canonical(project_path: &str, canonical: &Path) -> bool {
    Path::new(project_path) == canonical
        || canonical_existing_dir(Path::new(project_path))
            .map(|p| p == canonical)
            .unwrap_or(false)
}

fn path_is_inside_project(canonical_path: &Path, project_path: &str) -> bool {
    canonical_existing_dir(Path::new(project_path))
        .map(|project| canonical_path.starts_with(project))
        .unwrap_or(false)
}

pub async fn rename(id: &str, name: &str) -> Result<(), String> {
    let _guard = PROJECT_STORE_LOCK.lock().await;
    let mut projects = read_all_unlocked().await?;
    let p = projects
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("Projet introuvable")?;
    p.name = name.to_string();
    write_atomic(&projects).await
}

pub async fn delete(id: &str) -> Result<(), String> {
    let _guard = PROJECT_STORE_LOCK.lock().await;
    let mut projects = read_all_unlocked().await?;
    projects.retain(|p| p.id != id);
    for (i, p) in projects.iter_mut().enumerate() {
        p.order = i;
    }
    write_atomic(&projects).await
}

pub async fn reorder(ids: Vec<String>) -> Result<(), String> {
    let mut unique_ids = HashSet::with_capacity(ids.len().min(MAX_PROJECTS));
    if ids.len() > MAX_PROJECTS
        || ids
            .iter()
            .any(|id| !bounded_text(id, MAX_PROJECT_ID_BYTES) || !unique_ids.insert(id.as_str()))
    {
        return Err(PROJECT_STORE_UNAVAILABLE.to_string());
    }
    let _guard = PROJECT_STORE_LOCK.lock().await;
    let mut projects = read_all_unlocked().await?;
    for (i, id) in ids.iter().enumerate() {
        if let Some(p) = projects.iter_mut().find(|p| &p.id == id) {
            p.order = i;
        }
    }
    projects.sort_by_key(|p| p.order);
    write_atomic(&projects).await
}

fn validate_projects(projects: &[Project]) -> Result<(), String> {
    let mut unique_ids = HashSet::with_capacity(projects.len().min(MAX_PROJECTS));
    if projects.len() > MAX_PROJECTS
        || projects.iter().any(|project| {
            !bounded_text(&project.id, MAX_PROJECT_ID_BYTES)
                || !bounded_text(&project.name, MAX_PROJECT_NAME_BYTES)
                || !bounded_text(&project.path, MAX_PROJECT_PATH_BYTES)
                || !unique_ids.insert(project.id.as_str())
        })
    {
        return Err(PROJECT_STORE_UNAVAILABLE.to_string());
    }
    Ok(())
}

fn bounded_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && !value.bytes().any(|byte| matches!(byte, 0 | b'\r' | b'\n'))
}

#[cfg(test)]
#[path = "project_store_tests.rs"]
mod tests;
