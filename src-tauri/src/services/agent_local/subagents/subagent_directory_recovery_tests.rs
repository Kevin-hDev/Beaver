use super::{
    session_store, subagent_registry, subagent_task_change, subagent_working_dir,
};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn interrupted_directory_workspace_is_captured_before_cleanup() {
    let project = tempfile::tempdir().expect("project");
    std::fs::write(project.path().join("README.md"), "baseline\n").expect("seed");
    let parent = session_store::create_full("Parent", "model", "provider", false, None)
        .await
        .expect("parent");
    let mut child = session_store::create_full("Child", "model", "provider", false, None)
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
    .expect("prepare");
    tokio::fs::write(prepared.path().join("recovered.txt"), "recover\n")
        .await
        .expect("write change");
    subagent_registry::unregister(&child.id).await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    subagent_task_change::recover_and_remove_orphan(&saved)
        .await
        .expect("recover orphan");

    assert!(!prepared.path().exists());
    let change = super::subagent_change_store::load(&child.id)
        .await
        .expect("captured change");
    assert!(change.changed_paths.iter().any(|path| path.path == "recovered.txt"));
    let execution = super::subagent_directory_change::execution_id(&change)
        .expect("execution")
        .to_string();
    super::subagent_directory_workspace::remove_repository(&child.id, &execution)
        .await
        .expect("remove repository");
    let _ = super::subagent_change_store::remove(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
