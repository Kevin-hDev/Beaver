use super::{
    session_store, subagent_git_actions, subagent_git_run, subagent_registry,
    subagent_task_change, subagent_working_dir,
};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn coder_gets_an_isolated_workspace_for_a_directory_without_git() {
    let project = tempfile::tempdir().expect("project");
    std::fs::write(project.path().join("README.md"), "original\n").expect("seed project");
    let parent = session_store::create_full("Parent", "model", "provider", false, None)
        .await
        .expect("parent");
    let mut child = session_store::create_full(
        "Child",
        "model",
        "provider",
        false,
        None,
    )
    .await
    .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    child.working_dir = project.path().to_string_lossy().to_string();
    let run = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register");
    child.subagent_run_id = Some(run.run_id.clone());
    session_store::save(&child).await.expect("save child");

    let prepared = subagent_working_dir::resolve(
        None,
        &child.id,
        false,
        &run.run_id,
        &run.execution_id,
    )
    .await
    .expect("prepare non-git workspace");

    assert_ne!(prepared.path(), project.path());
    assert_eq!(
        std::fs::read_to_string(prepared.path().join("README.md")).unwrap(),
        "original\n"
    );
    subagent_registry::unregister(&child.id).await;
    subagent_working_dir::cleanup_owned(
        &child.id,
        &run.execution_id,
        prepared.worktree_path(),
    )
    .await;
    subagent_task_change::delete_empty_workspace(project.path(), &child.id, &run.execution_id)
        .await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}

#[tokio::test]
async fn directory_change_is_captured_and_applied_transactionally() {
    let project = tempfile::tempdir().expect("project");
    std::fs::write(project.path().join("README.md"), "original\n").expect("seed project");
    std::fs::write(project.path().join("delete.txt"), "delete\n").expect("seed delete");
    let mut parent = session_store::create_full(
        "Parent apply",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("parent");
    parent.working_dir = project.path().to_string_lossy().to_string();
    session_store::save(&parent).await.expect("save parent");
    let mut child = session_store::create_full(
        "Child apply",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    child.working_dir = project.path().to_string_lossy().to_string();
    let run = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register");
    child.subagent_run_id = Some(run.run_id.clone());
    session_store::save(&child).await.expect("save child");
    let prepared = subagent_working_dir::create_coder_worktree_for_test(
        project.path(),
        &child.id,
        &run.run_id,
        &run.execution_id,
    )
    .await
    .expect("prepare");
    tokio::fs::write(prepared.path().join("README.md"), "changed\n")
        .await
        .expect("modify");
    tokio::fs::write(prepared.path().join("added.txt"), "added\n")
        .await
        .expect("add");
    tokio::fs::remove_file(prepared.path().join("delete.txt"))
        .await
        .expect("delete");

    let initial_change = subagent_git_run::capture(
        project.path(),
        &child.id,
        &run.execution_id,
        prepared.path(),
    )
    .await
    .expect("capture")
    .expect("change");
    subagent_working_dir::cleanup_owned(
        &child.id,
        &run.execution_id,
        prepared.worktree_path(),
    )
    .await;
    assert_eq!(std::fs::read_to_string(project.path().join("README.md")).unwrap(), "original\n");

    std::fs::write(project.path().join("parent.txt"), "new parent state\n")
        .expect("advance directory");
    subagent_registry::unregister(&child.id).await;
    let next_run = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register correction");
    child.subagent_run_id = Some(next_run.run_id.clone());
    session_store::save(&child).await.expect("save correction run");
    let corrected = subagent_working_dir::create_coder_worktree_for_test(
        project.path(),
        &child.id,
        &next_run.run_id,
        &next_run.execution_id,
    )
    .await
    .expect("prepare correction");
    assert_eq!(std::fs::read_to_string(corrected.path().join("README.md")).unwrap(), "changed\n");
    assert_eq!(std::fs::read_to_string(corrected.path().join("parent.txt")).unwrap(), "new parent state\n");
    let change = subagent_git_run::capture(
        project.path(),
        &child.id,
        &next_run.execution_id,
        corrected.path(),
    )
    .await
    .expect("capture correction")
    .expect("corrected change");
    assert_eq!(change.id, initial_change.id);
    subagent_working_dir::cleanup_owned(
        &child.id,
        &next_run.execution_id,
        corrected.worktree_path(),
    )
    .await;

    subagent_git_actions::apply(project.path(), &parent.id, &child.id, &change.id)
        .await
        .expect("apply directory change");

    assert_eq!(std::fs::read_to_string(project.path().join("README.md")).unwrap(), "changed\n");
    assert_eq!(std::fs::read_to_string(project.path().join("added.txt")).unwrap(), "added\n");
    assert!(!project.path().join("delete.txt").exists());
    subagent_registry::unregister(&child.id).await;
    let _ = super::subagent_change_store::remove(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
