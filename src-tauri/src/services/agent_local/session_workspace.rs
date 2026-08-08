use super::types_session::AgentSession;
#[path = "session_workspace_name.rs"]
mod name;
#[path = "session_workspace_paths.rs"]
mod paths;
#[cfg(test)]
use name::SLUG_MAX_CHARS;
use name::{session_suffix, slugify, valid_date};
#[cfg(all(test, windows))]
use paths::relative_workspace_path_with;
use paths::{reject_symlinks, relative_workspace_path, validate_created_path, workspace_error};
use std::path::{Path, PathBuf};

pub struct SessionWorkspace {
    pub work: PathBuf,
    pub outputs: PathBuf,
}

pub async fn ensure(session: &AgentSession) -> Result<SessionWorkspace, String> {
    let base = crate::services::paths::data_dir().join("session-workspaces");
    let outputs_base = configured_outputs_base();
    if session.working_dir_managed && !session.working_dir.trim().is_empty() {
        return ensure_work_path(
            &base,
            Path::new(&session.working_dir),
            outputs_base.as_deref(),
        )
        .await;
    }
    let label = first_user_label(session)?;
    let date = session.created_at.format("%Y-%m-%d").to_string();
    ensure_layout(&base, outputs_base.as_deref(), &date, label, &session.id).await
}

async fn ensure_layout(
    base: &Path,
    outputs_base: Option<&Path>,
    date: &str,
    label: &str,
    session_id: &str,
) -> Result<SessionWorkspace, String> {
    super::session_store::validate_session_id(session_id)?;
    if !valid_date(date) {
        return Err(workspace_error());
    }
    let name = format!("{}-{}", slugify(label), session_suffix(session_id)?);
    let work = base.join(date).join(name).join("work");
    ensure_work_path(base, &work, outputs_base).await
}

async fn ensure_work_path(
    base: &Path,
    work: &Path,
    outputs_base: Option<&Path>,
) -> Result<SessionWorkspace, String> {
    let base = dunce::simplified(base);
    let work = dunce::simplified(work);
    let relative = relative_workspace_path(base, work)?;
    let mut components = relative.components();
    let date = normal_component(components.next())?;
    let name = normal_component(components.next())?;
    let work_component = normal_component(components.next())?;
    if work_component != "work" || components.next().is_some() {
        return Err(workspace_error());
    }
    let root = work.parent().ok_or_else(workspace_error)?;
    let outputs = match outputs_base {
        Some(outputs_base) => outputs_base.join(date).join(name).join("outputs"),
        None => root.join("outputs"),
    };
    let outputs_root = outputs_base.unwrap_or(base);
    reject_symlinks(outputs_root, &outputs)?;
    reject_symlinks(base, work)?;
    crate::services::private_store::ensure_private_dir_async(work.to_path_buf())
        .await
        .map_err(|_| workspace_error())?;
    crate::services::private_store::ensure_private_dir_async(outputs.clone())
        .await
        .map_err(|_| workspace_error())?;
    validate_created_path(base, work)?;
    validate_created_path(outputs_root, &outputs)?;
    Ok(SessionWorkspace {
        work: work.to_path_buf(),
        outputs,
    })
}

fn normal_component(
    component: Option<std::path::Component<'_>>,
) -> Result<&std::ffi::OsStr, String> {
    match component {
        Some(std::path::Component::Normal(value)) if !value.to_string_lossy().contains('\0') => {
            Ok(value)
        }
        _ => Err(workspace_error()),
    }
}

fn configured_outputs_base() -> Option<PathBuf> {
    let value = crate::services::config::session_outputs_directory()?;
    crate::models::config::existing_optional_directory(value.to_string_lossy().as_ref())
}

pub(crate) fn access_roots_for(candidate: &Path) -> Vec<PathBuf> {
    let base = crate::services::paths::data_dir().join("session-workspaces");
    let Some(base) = dunce::canonicalize(base).ok().filter(|path| path.is_dir()) else {
        return Vec::new();
    };
    let Some(candidate) = dunce::canonicalize(candidate)
        .ok()
        .filter(|path| path.is_dir())
    else {
        return Vec::new();
    };
    let Ok(relative) = candidate.strip_prefix(&base) else {
        return Vec::new();
    };
    let components = relative.components().collect::<Vec<_>>();
    if components.len() < 3
        || normal_component(components.first().copied()).is_err()
        || normal_component(components.get(1).copied()).is_err()
        || normal_component(components.get(2).copied()).ok() != Some(std::ffi::OsStr::new("work"))
    {
        return Vec::new();
    }
    let work = base
        .join(components[0].as_os_str())
        .join(components[1].as_os_str())
        .join("work");
    let Some(work) = dunce::canonicalize(work)
        .ok()
        .filter(|path| candidate.starts_with(path))
    else {
        return Vec::new();
    };
    let outputs = configured_outputs_base()
        .map(|root| {
            root.join(components[0].as_os_str())
                .join(components[1].as_os_str())
                .join("outputs")
        })
        .or_else(|| work.parent().map(|root| root.join("outputs")));
    let mut roots = vec![work];
    if let Some(output) = outputs
        .and_then(|path| dunce::canonicalize(path).ok())
        .filter(|path| path.is_dir())
    {
        roots.push(output);
    }
    roots
}

fn first_user_label(session: &AgentSession) -> Result<&str, String> {
    let Some(message) = session
        .messages
        .iter()
        .find(|message| message.role == "user")
    else {
        return Err(workspace_error());
    };
    if !message.content.trim().is_empty() {
        return Ok(&message.content);
    }
    Ok(message
        .files
        .first()
        .map(|file| file.name.as_str())
        .unwrap_or(&session.name))
}

#[cfg(test)]
#[path = "session_workspace_tests.rs"]
mod tests;
