#![cfg(unix)]

use super::{session_store, subagent_git_run, subagent_worktree};
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn temporary_capture_does_not_run_project_commit_hooks() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let hook = repo.path().join(".git/hooks/pre-commit");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").expect("write hook");
    let mut permissions = std::fs::metadata(&hook).expect("hook metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&hook, permissions).expect("hook permissions");

    let parent = session_store::create_full(
        "Parent",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("parent");
    let mut child = session_store::create_full(
        "Child",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    session_store::save(&child).await.expect("save child");
    let execution = uuid::Uuid::new_v4().to_string();
    let worktree = subagent_worktree::create_for_execution(
        repo.path(),
        &child.id,
        &execution,
    )
    .await
    .expect("worktree");
    tokio::fs::write(worktree.join("captured.txt"), "captured\n")
        .await
        .expect("write change");

    let captured = subagent_git_run::capture(repo.path(), &child.id, &execution, &worktree)
        .await
        .expect("capture");

    assert!(captured.is_some());
    subagent_worktree::remove_owned(&worktree.to_string_lossy(), &child.id, &execution)
        .await
        .expect("cleanup worktree");
    let _ = super::subagent_change_store::remove(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
