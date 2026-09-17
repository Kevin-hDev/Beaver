use super::{
    session_store, subagent_git_actions, subagent_git_run, subagent_registry,
    subagent_working_dir,
};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn empty_directory_can_be_prepared_for_a_coder() {
    let project = tempfile::tempdir().expect("project");
    let child_id = uuid::Uuid::new_v4().to_string();
    let execution_id = uuid::Uuid::new_v4().to_string();

    let worktree = super::subagent_directory_workspace::create(
        project.path(),
        &child_id,
        &execution_id,
    )
    .await
    .expect("empty workspace");

    assert!(worktree.is_dir());
    super::subagent_worktree::remove_owned(&worktree.to_string_lossy(), &child_id, &execution_id)
        .await
        .expect("remove worktree");
    super::subagent_directory_workspace::remove_repository(&child_id, &execution_id)
        .await
        .expect("remove repository");
}

#[tokio::test]
async fn directory_conflict_preserves_user_files_and_can_be_discarded() {
    let project = tempfile::tempdir().expect("project");
    std::fs::write(project.path().join("README.md"), "baseline\n").expect("seed");
    let mut parent = session_store::create_full(
        "Parent",
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
    tokio::fs::write(prepared.path().join("README.md"), "child\n")
        .await
        .expect("child edit");
    let change = subagent_git_run::capture(
        project.path(),
        &child.id,
        &run.execution_id,
        prepared.path(),
    )
    .await
    .expect("capture")
    .expect("change");
    let mut running_child = session_store::get(&child.id).await.expect("running child");
    running_child.working_dir = prepared.path().to_string_lossy().to_string();
    session_store::save(&running_child).await.expect("save worktree path");
    subagent_working_dir::cleanup_owned(
        &child.id,
        &run.execution_id,
        prepared.worktree_path(),
    )
    .await;
    let (_, patch, _) = subagent_git_actions::inspect(
        project.path(),
        &parent.id,
        &child.id,
        &change.id,
    )
    .await
    .expect("inspect");
    assert!(patch.contains("child"));
    std::fs::write(project.path().join("README.md"), "user\n").expect("user edit");

    assert!(subagent_git_actions::apply(project.path(), &parent.id, &child.id, &change.id)
        .await
        .is_err());
    assert_eq!(std::fs::read_to_string(project.path().join("README.md")).unwrap(), "user\n");
    let conflict = super::subagent_change_store::load(&child.id).await.expect("conflict");
    assert_eq!(conflict.status, super::types_subagent_change::SubagentChangeStatus::Conflict);
    subagent_git_actions::discard(project.path(), &parent.id, &child.id, &change.id)
        .await
        .expect("discard");
    let repository = super::subagent_directory_workspace::repository_path(
        &child.id,
        &run.execution_id,
    )
    .expect("repository");
    assert!(!repository.exists());
    subagent_registry::unregister(&child.id).await;
    let _ = super::subagent_change_store::remove(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
