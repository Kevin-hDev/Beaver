use crate::services::agent_local::{project_store, session_store};
use std::path::{Path, PathBuf};

pub(crate) struct ResolvedWorkingDir {
    pub path: PathBuf,
    pub outputs_dir: Option<PathBuf>,
}

pub(crate) async fn resolve_for_session(
    session_id: &str,
    incoming: Option<&str>,
) -> Result<ResolvedWorkingDir, String> {
    let session = session_store::get(session_id)
        .await
        .map_err(|_| "Session introuvable".to_string())?;
    let project_dir = match session.project_id.as_deref() {
        Some(project_id) => match project_path_for_id(project_id).await? {
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
    let stored_dir = if project_dir.is_none()
        && incoming_dir.is_none()
        && !session.working_dir_managed
        && !is_home_directory(&session.working_dir)
    {
        canonical_optional_dir(Some(&session.working_dir))?
    } else {
        None
    };
    if let Some(resolved) = choose_project_root(project_dir, incoming_dir, stored_dir) {
        session_store::update_working_dir(session_id, resolved.path.to_string_lossy().as_ref())
            .await
            .map_err(|_| "Impossible d'enregistrer le dossier de travail.".to_string())?;
        return Ok(resolved);
    }

    let workspace = crate::services::agent_local::session_workspace::ensure(&session).await?;
    session_store::set_managed_working_dir(session_id, workspace.work.to_string_lossy().as_ref())
        .await
        .map_err(|_| "Impossible d'enregistrer le dossier de travail.".to_string())?;
    Ok(ResolvedWorkingDir {
        path: workspace.work,
        outputs_dir: Some(workspace.outputs),
    })
}

pub(crate) async fn resolve_existing_for_session(
    session_id: &str,
    incoming: Option<&str>,
) -> Result<Option<ResolvedWorkingDir>, String> {
    let session = session_store::get(session_id)
        .await
        .map_err(|_| "Session introuvable".to_string())?;
    let project_dir = match session.project_id.as_deref() {
        Some(project_id) => project_path_for_id(project_id)
            .await?
            .map(|path| canonical_dir(&path))
            .transpose()?,
        None => None,
    };
    let incoming_dir = if project_dir.is_none() {
        canonical_optional_dir(incoming)?
    } else {
        None
    };
    let stored_dir = if project_dir.is_none()
        && incoming_dir.is_none()
        && !is_home_directory(&session.working_dir)
    {
        canonical_optional_dir(Some(&session.working_dir))
            .ok()
            .flatten()
    } else {
        None
    };
    Ok(choose_project_root(project_dir, incoming_dir, stored_dir))
}

pub(crate) async fn project_path_for_id(project_id: &str) -> Result<Option<String>, String> {
    Ok(project_store::find(project_id)
        .await?
        .map(|project| project.path))
}

fn canonical_dir(input: &str) -> Result<ResolvedWorkingDir, String> {
    let path = canonical_existing_dir(input)?;
    let path = crate::services::agent_local::directory_access::ensure_allowed(&path)?;
    Ok(ResolvedWorkingDir {
        path,
        outputs_dir: None,
    })
}

fn canonical_existing_dir(input: &str) -> Result<PathBuf, String> {
    let path = Path::new(input);
    if !path.is_dir() {
        return Err("Répertoire introuvable".to_string());
    }
    dunce::canonicalize(path).map_err(|_| "Répertoire inaccessible".to_string())
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

fn is_home_directory(input: &str) -> bool {
    let input = input.trim();
    if input.is_empty() {
        return false;
    }
    let Some(home) = dirs::home_dir() else {
        return false;
    };
    canonical_existing_dir(input)
        .ok()
        .and_then(|candidate| dunce::canonicalize(home).ok().map(|home| candidate == home))
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "agent_working_dir_tests.rs"]
mod tests;
