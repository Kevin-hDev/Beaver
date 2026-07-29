use crate::services::agent_local::{project_store, session_store};
use std::path::{Path, PathBuf};

pub(crate) struct ResolvedWorkingDir {
    pub path: PathBuf,
}

pub(crate) async fn resolve_for_session(
    session_id: &str,
    incoming: Option<&str>,
) -> Result<ResolvedWorkingDir, String> {
    let session = session_store::get(session_id)
        .await
        .map_err(|_| "Session introuvable".to_string())?;
    let project_dir = match session.project_id.as_deref() {
        Some(project_id) => match project_path_for_id(project_id).await {
            Some(path) => Some(canonical_dir(&path)?),
            None => None,
        },
        None => None,
    };
    let incoming_dir = if project_dir.is_none() {
        canonical_optional_dir(incoming)?
    } else {
        None
    };
    let stored_dir = if project_dir.is_none() && incoming_dir.is_none() {
        canonical_optional_dir(Some(&session.working_dir))?
    } else {
        None
    };
    if let Some(resolved) = choose_project_root(project_dir, incoming_dir, stored_dir) {
        return Ok(resolved);
    }

    let path = dirs::home_dir()
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| "Répertoire de travail introuvable".to_string())?;
    Ok(ResolvedWorkingDir {
        path: path.canonicalize().unwrap_or(path),
    })
}

pub(crate) async fn project_path_for_id(project_id: &str) -> Option<String> {
    project_store::list()
        .await
        .ok()?
        .into_iter()
        .find(|project| project.id == project_id)
        .map(|project| project.path)
}

fn canonical_dir(input: &str) -> Result<ResolvedWorkingDir, String> {
    let path = Path::new(input);
    if !path.is_dir() {
        return Err("Répertoire introuvable".to_string());
    }
    let path = path
        .canonicalize()
        .map_err(|_| "Répertoire inaccessible".to_string())?;
    Ok(ResolvedWorkingDir { path })
}

fn canonical_optional_dir(input: Option<&str>) -> Result<Option<ResolvedWorkingDir>, String> {
    input
        .map(str::trim)
        .filter(|dir| !dir.is_empty())
        .map(canonical_dir)
        .transpose()
}

fn choose_project_root(
    project_dir: Option<ResolvedWorkingDir>,
    incoming_dir: Option<ResolvedWorkingDir>,
    stored_dir: Option<ResolvedWorkingDir>,
) -> Option<ResolvedWorkingDir> {
    project_dir.or(incoming_dir).or(stored_dir)
}

#[cfg(test)]
mod tests {
    use super::{canonical_dir, canonical_optional_dir, choose_project_root, ResolvedWorkingDir};

    #[test]
    fn canonicalizes_existing_dir() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("nested");
        std::fs::create_dir_all(&nested).expect("nested");

        let resolved = canonical_dir(&nested.join(".").to_string_lossy()).expect("resolved");

        assert_eq!(
            resolved.path,
            std::fs::canonicalize(&nested).expect("canonical")
        );
    }

    #[test]
    fn rejects_a_missing_stored_root_instead_of_falling_back() {
        let missing = "/definitely/missing/beaver-project-root";

        assert!(canonical_optional_dir(Some(missing)).is_err());
    }

    #[test]
    fn project_wins_over_incoming_and_stored_directories() {
        let temp = tempfile::tempdir().expect("tempdir");
        let project = temp.path().join("project");
        let incoming = temp.path().join("incoming");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&project).expect("project");
        std::fs::create_dir_all(&incoming).expect("incoming");
        std::fs::create_dir_all(&outside).expect("outside");

        let resolved = choose_project_root(
            Some(ResolvedWorkingDir {
                path: project.clone(),
            }),
            Some(ResolvedWorkingDir { path: incoming }),
            Some(ResolvedWorkingDir { path: outside }),
        )
        .expect("resolved");

        assert_eq!(resolved.path, project);
    }

    #[test]
    fn project_root_wins_over_a_stored_subdirectory() {
        let temp = tempfile::tempdir().expect("tempdir");
        let project = temp.path().join("project");
        let nested = project.join("nested");
        std::fs::create_dir_all(&nested).expect("nested");

        let resolved = choose_project_root(
            Some(ResolvedWorkingDir {
                path: project.clone(),
            }),
            None,
            Some(ResolvedWorkingDir { path: nested }),
        )
        .expect("resolved");

        assert_eq!(resolved.path, project);
    }

    #[test]
    fn projectless_session_prefers_incoming_then_stored_directory() {
        let incoming = ResolvedWorkingDir {
            path: std::path::PathBuf::from("/incoming"),
        };
        let stored = ResolvedWorkingDir {
            path: std::path::PathBuf::from("/stored"),
        };

        let selected =
            choose_project_root(None, Some(incoming), Some(stored)).expect("incoming directory");

        assert_eq!(selected.path, std::path::PathBuf::from("/incoming"));
    }
}
