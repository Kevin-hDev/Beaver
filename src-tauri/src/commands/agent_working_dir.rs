use crate::services::agent_local::{project_store, session_store};
use std::path::{Path, PathBuf};

pub(crate) struct ResolvedWorkingDir {
    pub path: PathBuf,
}

pub(crate) async fn resolve_for_session(
    session_id: &str,
    incoming: Option<&str>,
) -> Result<ResolvedWorkingDir, String> {
    if let Some(dir) = incoming.map(str::trim).filter(|dir| !dir.is_empty()) {
        return canonical_dir(dir);
    }

    let session = session_store::get(session_id)
        .await
        .map_err(|_| "Session introuvable".to_string())?;
    let project_dir = match session.project_id.as_deref() {
        Some(project_id) => project_path_for_id(project_id)
            .await
            .and_then(|path| canonical_dir_if_valid(&path)),
        None => None,
    };
    if let Some(resolved) =
        choose_stored_or_project(canonical_dir_if_valid(&session.working_dir), project_dir)
    {
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

fn canonical_dir_if_valid(input: &str) -> Option<ResolvedWorkingDir> {
    let dir = input.trim();
    if dir.is_empty() {
        return None;
    }
    canonical_dir(dir).ok()
}

fn choose_stored_or_project(
    stored_dir: Option<ResolvedWorkingDir>,
    project_dir: Option<ResolvedWorkingDir>,
) -> Option<ResolvedWorkingDir> {
    if let Some(stored) = stored_dir {
        if project_dir
            .as_ref()
            .map(|project| stored.path.starts_with(&project.path))
            .unwrap_or(true)
        {
            return Some(stored);
        }
    }
    project_dir
}

#[cfg(test)]
mod tests {
    use super::{canonical_dir_if_valid, choose_stored_or_project, ResolvedWorkingDir};

    #[test]
    fn ignores_empty_dir() {
        assert!(canonical_dir_if_valid("").is_none());
        assert!(canonical_dir_if_valid("   ").is_none());
    }

    #[test]
    fn canonicalizes_existing_dir() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("nested");
        std::fs::create_dir_all(&nested).expect("nested");

        let resolved =
            canonical_dir_if_valid(&nested.join(".").to_string_lossy()).expect("resolved");

        assert_eq!(
            resolved.path,
            std::fs::canonicalize(&nested).expect("canonical")
        );
    }

    #[test]
    fn project_wins_when_stored_dir_is_outside_project() {
        let temp = tempfile::tempdir().expect("tempdir");
        let project = temp.path().join("project");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&project).expect("project");
        std::fs::create_dir_all(&outside).expect("outside");

        let resolved = choose_stored_or_project(
            Some(ResolvedWorkingDir { path: outside }),
            Some(ResolvedWorkingDir {
                path: project.clone(),
            }),
        )
        .expect("resolved");

        assert_eq!(resolved.path, project);
    }
}
