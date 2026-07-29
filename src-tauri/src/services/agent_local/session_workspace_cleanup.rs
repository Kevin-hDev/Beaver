use super::AgentSession;
use std::path::{Path, PathBuf};

pub(super) async fn remove_for_deleted_session(
    session: &AgentSession,
) -> Result<(), String> {
    if !owns_managed_workspace(session) {
        return Ok(());
    }
    let base = crate::services::paths::data_dir().join("session-workspaces");
    remove_managed_root(&base, Path::new(&session.working_dir)).await
}

fn owns_managed_workspace(session: &AgentSession) -> bool {
    owner_fields_are_valid(
        session.working_dir_managed,
        session.parent_session_id.as_deref(),
        session.clone_parent_session_id.as_deref(),
        &session.working_dir,
    )
}

fn owner_fields_are_valid(
    managed: bool,
    parent_session_id: Option<&str>,
    clone_parent_session_id: Option<&str>,
    working_dir: &str,
) -> bool {
    managed
        && parent_session_id.is_none()
        && clone_parent_session_id.is_none()
        && !working_dir.trim().is_empty()
}

async fn remove_managed_root(base: &Path, work: &Path) -> Result<(), String> {
    let Ok((date_dir, root)) = validated_root(base, work) else {
        return Ok(());
    };
    if std::fs::symlink_metadata(&root).is_err() {
        return Ok(());
    }
    super::reject_symlinks(base, &root)?;
    let canonical_base = base.canonicalize().map_err(|_| super::workspace_error())?;
    let canonical_date = date_dir
        .canonicalize()
        .map_err(|_| super::workspace_error())?;
    let canonical_root = root.canonicalize().map_err(|_| super::workspace_error())?;
    if canonical_root.parent() != Some(canonical_date.as_path())
        || !canonical_root.starts_with(&canonical_base)
    {
        return Err(super::workspace_error());
    }
    tokio::fs::remove_dir_all(&canonical_root)
        .await
        .map_err(|_| super::workspace_error())?;
    remove_empty_date_dir(&date_dir).await;
    Ok(())
}

fn validated_root(base: &Path, work: &Path) -> Result<(PathBuf, PathBuf), String> {
    let relative = work
        .strip_prefix(base)
        .map_err(|_| super::workspace_error())?;
    let mut components = relative.components();
    let date = super::normal_component(components.next())?;
    let name = super::normal_component(components.next())?;
    let leaf = super::normal_component(components.next())?;
    if !super::valid_date(&date.to_string_lossy())
        || leaf != "work"
        || components.next().is_some()
    {
        return Err(super::workspace_error());
    }
    let date_dir = base.join(date);
    Ok((date_dir.clone(), date_dir.join(name)))
}

async fn remove_empty_date_dir(date_dir: &Path) {
    let Ok(mut entries) = tokio::fs::read_dir(date_dir).await else {
        return;
    };
    if entries.next_entry().await.ok().flatten().is_none() {
        let _ = tokio::fs::remove_dir(date_dir).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn deleting_an_owner_removes_only_its_managed_root() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().join("session-workspaces");
        let root = base.join("2026-07-29").join("analyse-deadbeef");
        let work = root.join("work");
        let outputs = root.join("outputs");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&outputs).unwrap();
        std::fs::write(outputs.join("report.md"), "kept until deletion").unwrap();

        remove_managed_root(&base, &work).await.unwrap();

        assert!(!root.exists());
    }

    #[tokio::test]
    async fn a_path_outside_the_managed_base_is_never_removed() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().join("session-workspaces");
        let outside = temp.path().join("selected");
        let work = outside.join("work");
        std::fs::create_dir_all(&work).unwrap();

        remove_managed_root(&base, &work).await.unwrap();

        assert!(outside.exists());
    }

    #[test]
    fn explicit_directories_and_shared_children_are_not_owned() {
        assert!(!owner_fields_are_valid(false, None, None, "/selected"));
        assert!(!owner_fields_are_valid(true, Some("parent"), None, "/shared"));
        assert!(!owner_fields_are_valid(
            true,
            None,
            Some("clone-parent"),
            "/shared"
        ));
    }

    #[test]
    fn a_top_level_managed_session_owns_its_workspace() {
        assert!(owner_fields_are_valid(true, None, None, "/managed/work"));
    }
}
