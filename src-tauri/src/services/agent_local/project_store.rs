use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

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

async fn read_all() -> Vec<Project> {
    let path = projects_path();
    let data = match tokio::fs::read_to_string(&path).await {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&data).unwrap_or_default()
}

async fn write_atomic(projects: &[Project]) -> Result<(), String> {
    let path = projects_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_string_pretty(projects).map_err(|e| format!("Serialize: {e}"))?;
    tokio::fs::write(&tmp, &data)
        .await
        .map_err(|e| format!("Write tmp: {e}"))?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| format!("Rename: {e}"))?;
    Ok(())
}

pub async fn list() -> Result<Vec<Project>, String> {
    let mut projects = read_all().await;
    projects.sort_by_key(|p| p.order);
    Ok(projects)
}

pub async fn add(path: &str) -> Result<Project, String> {
    let canonical = canonical_existing_dir(Path::new(path))?;
    let canonical_path = canonical.to_string_lossy().to_string();
    let mut projects = read_all().await;
    if let Some(existing) = projects
        .iter()
        .find(|p| project_matches_canonical(&p.path, &canonical))
    {
        return Ok(existing.clone());
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
    let projects = read_all().await;
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
    let canonical = std::fs::canonicalize(path).map_err(|_| "Dossier introuvable".to_string())?;
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
    let mut projects = read_all().await;
    let p = projects
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("Projet introuvable")?;
    p.name = name.to_string();
    write_atomic(&projects).await
}

pub async fn delete(id: &str) -> Result<(), String> {
    let mut projects = read_all().await;
    projects.retain(|p| p.id != id);
    for (i, p) in projects.iter_mut().enumerate() {
        p.order = i;
    }
    write_atomic(&projects).await
}

pub async fn reorder(ids: Vec<String>) -> Result<(), String> {
    let mut projects = read_all().await;
    for (i, id) in ids.iter().enumerate() {
        if let Some(p) = projects.iter_mut().find(|p| &p.id == id) {
            p.order = i;
        }
    }
    projects.sort_by_key(|p| p.order);
    write_atomic(&projects).await
}

#[cfg(test)]
mod tests {
    use super::{canonical_existing_dir, path_is_inside_project, project_matches_canonical};

    #[test]
    fn canonical_existing_dir_normalizes_dot_segments() {
        let tmp = tempfile::tempdir().expect("temp");
        let nested = tmp.path().join("nested");
        std::fs::create_dir_all(&nested).expect("nested");

        let canonical = canonical_existing_dir(&nested.join(".")).expect("canonical");

        assert_eq!(canonical, std::fs::canonicalize(&nested).expect("expected"));
    }

    #[test]
    fn project_match_accepts_equivalent_path() {
        let tmp = tempfile::tempdir().expect("temp");
        let canonical = std::fs::canonicalize(tmp.path()).expect("canonical");
        let equivalent = tmp.path().join(".");

        assert!(project_matches_canonical(
            &equivalent.to_string_lossy(),
            &canonical
        ));
    }

    #[test]
    fn inside_project_allows_child_and_rejects_sibling() {
        let tmp = tempfile::tempdir().expect("temp");
        let project = tmp.path().join("project");
        let child = project.join("child");
        let sibling = tmp.path().join("sibling");
        std::fs::create_dir_all(&child).expect("child");
        std::fs::create_dir_all(&sibling).expect("sibling");

        let child = std::fs::canonicalize(child).expect("canonical child");
        let sibling = std::fs::canonicalize(sibling).expect("canonical sibling");

        assert!(path_is_inside_project(&child, &project.to_string_lossy()));
        assert!(!path_is_inside_project(
            &sibling,
            &project.to_string_lossy()
        ));
    }
}
