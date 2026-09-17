use super::{
    session_store, subagent_hidden_reports, subagent_panic_supervisor, subagent_registry,
    subagent_status,
};
use tokio_util::sync::CancellationToken;

struct DropProbe(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[tokio::test]
async fn aborting_the_guard_cannot_detach_the_owned_subagent() {
    let dropped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let probe = DropProbe(std::sync::Arc::clone(&dropped));
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let guard = tokio::spawn(subagent_panic_supervisor::run_guarded(
        async move {
            let _probe = probe;
            let _ = started_tx.send(());
            std::future::pending::<()>().await;
        },
        || async {},
    ));
    started_rx.await.expect("owned subagent starts");

    guard.abort();
    let _ = guard.await;
    tokio::task::yield_now().await;

    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

async fn session(name: &str) -> super::types_session::AgentSession {
    session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create session")
}

#[tokio::test]
async fn panic_persists_generic_failure_and_leaves_no_registry_ghost() {
    let parent = session("Parent panic").await;
    let mut child = session("Geminitor").await;
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    let registered =
        subagent_registry::register_execution(&parent.id, &child.id, CancellationToken::new())
            .await
            .expect("register child");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save run id");
    let mut signal = subagent_registry::subscribe_for_parent(&parent.id)
        .await
        .expect("subscribe signal");
    let parent_id = parent.id.clone();
    let child_id = child.id.clone();
    let run_id = registered.run_id;
    let execution_id = registered.execution_id;

    subagent_panic_supervisor::run_guarded(
        async { panic!("internal panic must stay private") },
        move || async move {
            subagent_panic_supervisor::recover_panicked_completion(
                &parent_id,
                &child_id,
                "explorer",
                &run_id,
                &execution_id,
                None,
                None,
            )
            .await;
        },
    )
    .await;

    signal.changed().await.expect("panic completion signal");
    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    let saved_child = session_store::get(&child.id).await.expect("saved child");
    assert_eq!(
        saved_child.subagent_status.as_deref(),
        Some(subagent_status::FAILED)
    );
    let reports = subagent_hidden_reports::peek_reports(&parent.id).await;
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].summary,
        subagent_panic_supervisor::SUBAGENT_PANIC_SUMMARY
    );
    assert!(!reports[0].summary.contains("internal panic"));
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn panic_capture_failure_keeps_worktree_but_releases_registry() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit().await;
    let parent = session("Coder panic parent").await;
    let mut child = session_store::create_full(
        "Coder panic",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("coder");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.working_dir = repo.path().to_string_lossy().into_owned();
    let registered =
        subagent_registry::register_execution(&parent.id, &child.id, CancellationToken::new())
            .await
            .expect("register coder");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save coder");
    let worktree = super::subagent_worktree::create_for_execution(
        repo.path(),
        &child.id,
        &registered.execution_id,
    )
    .await
    .expect("worktree");
    child.subagent_worktree = Some(worktree.to_string_lossy().into_owned());
    session_store::save(&child).await.expect("save worktree");
    tokio::fs::write(worktree.join("unfinished.txt"), "keep\n")
        .await
        .expect("unfinished change");
    super::subagent_task_change::fail_next_capture_for_child(&child.id).await;

    let recovered = subagent_panic_supervisor::recover_panicked_completion(
        &parent.id,
        &child.id,
        "coder",
        &registered.run_id,
        &registered.execution_id,
        child.subagent_worktree.as_deref(),
        None,
    )
    .await;

    assert!(recovered);
    assert!(worktree.is_dir());
    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    let saved = session_store::get(&child.id).await.expect("saved coder");
    assert_eq!(saved.subagent_worktree, child.subagent_worktree);

    let _ = super::subagent_worktree::remove_owned(
        &worktree.to_string_lossy(),
        &child.id,
        &registered.execution_id,
    )
    .await;
    if let Ok(branch) = super::subagent_worktree::branch_for_execution(&registered.execution_id) {
        let _ = super::subagent_git_command::delete_branch(repo.path(), &branch).await;
    }
    session_store::delete_one(&child.id)
        .await
        .expect("delete coder");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
