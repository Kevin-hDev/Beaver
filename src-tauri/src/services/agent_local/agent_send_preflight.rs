use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PrepareAgentSend {
    Ready,
    Forbidden { allowed_paths: Vec<String> },
    Missing {
        missing_path: String,
        nearest_parent: String,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissingDirectoryAction {
    Switch,
    Create,
}

pub async fn prepare(
    session_id: &str,
    incoming: Option<&str>,
) -> Result<PrepareAgentSend, String> {
    let session = super::session_store::get(session_id)
        .await
        .map_err(|_| generic_error())?;
    let project_path = match session.project_id.as_deref() {
        Some(project_id) => super::project_store::find(project_id)
            .await
            .map(|project| PathBuf::from(project.path)),
        None => None,
    };
    let expected = project_path.or_else(|| incoming
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            (!session.working_dir_managed)
                .then(|| non_empty_path(&session.working_dir))
                .flatten()
        }));
    let Some(expected) = expected else {
        return Ok(PrepareAgentSend::Ready);
    };
    validate_path(&expected)?;
    let access = super::directory_access::decision(&expected).map_err(|_| generic_error())?;
    if !access.allowed {
        return Ok(PrepareAgentSend::Forbidden {
            allowed_paths: access.allowed_paths,
        });
    }
    if expected.is_dir() {
        return Ok(PrepareAgentSend::Ready);
    }
    let parent = nearest_existing_parent(&expected)?;
    Ok(PrepareAgentSend::Missing {
        missing_path: path_text(&expected)?,
        nearest_parent: path_text(&parent)?,
    })
}

pub async fn resolve(
    session_id: &str,
    missing_path: &str,
    action: MissingDirectoryAction,
) -> Result<String, String> {
    let target = PathBuf::from(missing_path);
    validate_path(&target)?;
    if !super::directory_access::decision(&target)
        .map_err(|_| generic_error())?
        .allowed
    {
        return Err(generic_error());
    }
    let session = super::session_store::get(session_id)
        .await
        .map_err(|_| generic_error())?;
    if !is_session_path(&session, &target).await || target.exists() {
        return Err(generic_error());
    }
    let resolved = match action {
        MissingDirectoryAction::Switch => nearest_existing_parent(&target)?,
        MissingDirectoryAction::Create => {
            tokio::fs::create_dir_all(&target)
                .await
                .map_err(|_| generic_error())?;
            dunce::canonicalize(&target).map_err(|_| generic_error())?
        }
    };
    let text = path_text(&resolved)?;
    match action {
        MissingDirectoryAction::Switch => {
            let project = super::project_store::add(&text)
                .await
                .map_err(|_| generic_error())?;
            super::session_store::switch_working_dir_to_project(session_id, &text, &project.id)
                .await
        }
        MissingDirectoryAction::Create => {
            super::session_store::update_working_dir(session_id, &text).await
        }
    }
    .map_err(|_| generic_error())?;
    Ok(text)
}

async fn is_session_path(session: &super::types_session::AgentSession, target: &Path) -> bool {
    if non_empty_path(&session.working_dir).as_deref() == Some(target) {
        return true;
    }
    let Some(project_id) = session.project_id.as_deref() else {
        return false;
    };
    super::project_store::find(project_id)
        .await
        .is_some_and(|project| Path::new(&project.path) == target)
}

fn non_empty_path(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    (!value.is_empty()).then(|| PathBuf::from(value))
}

fn nearest_existing_parent(path: &Path) -> Result<PathBuf, String> {
    let mut candidate = path.parent();
    while let Some(parent) = candidate {
        if parent.is_dir() {
            return dunce::canonicalize(parent).map_err(|_| generic_error());
        }
        candidate = parent.parent();
    }
    Err(generic_error())
}

fn validate_path(path: &Path) -> Result<(), String> {
    let text = path.to_str().ok_or_else(generic_error)?;
    if !path.is_absolute()
        || text.contains('\0')
        || text.chars().count() > 4_096
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(generic_error());
    }
    Ok(())
}

fn path_text(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(ToString::to_string)
        .ok_or_else(generic_error)
}

fn generic_error() -> String {
    "Impossible de préparer le dossier de cette session".to_string()
}
