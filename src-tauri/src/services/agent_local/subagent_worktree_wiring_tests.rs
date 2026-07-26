use super::{session_store, subagent_registry, subagent_working_dir};
use tokio_util::sync::CancellationToken;

#[test]
fn working_dir_is_prepared_before_the_internal_loop() {
    let task = include_str!("subagent_task.rs");
    let stream = include_str!("subagent_task_stream.rs");
    let spawn = include_str!("subagent_spawn_channel.rs");
    let resolve = task.find("subagent_working_dir::resolve").expect("resolve call");
    let loop_start = task.find("loop {").expect("internal loop");
    let task_ownership = task.find("owns_execution").expect("task ownership check");
    let spawn_ownership = spawn.find("owns_execution").expect("spawn ownership check");
    let spawn_task = spawn.find("subagent_task::run").expect("spawn task call");

    assert!(task_ownership < resolve);
    assert!(resolve < loop_start);
    assert!(spawn_ownership < spawn_task);
    assert!(task.contains("working_dir.clone(),"));
    assert!(!stream.contains("subagent_working_dir::resolve"));
    assert!(!spawn.contains(".ok().flatten()"));
}

#[tokio::test]
async fn concurrent_preparation_allows_exactly_one_worktree_claim() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let parent = session_store::create_full("Parent claim", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Coder claim", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    child.subagent_status = Some(super::subagent_status::RUNNING.into());
    let registered = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register execution");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save child");

    let first = subagent_working_dir::create_coder_worktree_for_test(
        repo.path(),
        &child.id,
        &registered.run_id,
        &registered.execution_id,
    );
    let second = subagent_working_dir::create_coder_worktree_for_test(
        repo.path(),
        &child.id,
        &registered.run_id,
        &registered.execution_id,
    );
    let (first, second) = tokio::join!(first, second);
    let successes = [first.as_ref().ok(), second.as_ref().ok()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    assert_eq!(successes.len(), 1);
    assert!(successes[0].path().is_dir());
    let owned_path = successes[0].worktree_path().map(ToString::to_string);
    drop(successes);
    subagent_registry::unregister(&child.id).await;
    subagent_working_dir::cleanup_owned(
        &child.id,
        &registered.execution_id,
        owned_path.as_deref(),
    )
    .await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
