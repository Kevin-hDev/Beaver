use futures_util::FutureExt;
use std::future::Future;
use std::panic::AssertUnwindSafe;

pub const SUBAGENT_PANIC_SUMMARY: &str = "Le sous-agent n'a pas pu terminer correctement.";

pub async fn run_guarded<F, Recover, RecoverFuture>(future: F, recover: Recover)
where
    F: Future<Output = ()> + Send + 'static,
    Recover: FnOnce() -> RecoverFuture,
    RecoverFuture: Future<Output = ()>,
{
    // La capture de panique reste dans la tâche propriétaire : une annulation
    // du superviseur ne doit jamais détacher le sous-agent qu'il possède.
    if AssertUnwindSafe(future).catch_unwind().await.is_err() {
        recover().await;
    }
}

pub async fn recover_panicked_completion(
    parent_session_id: &str,
    child_session_id: &str,
    subagent_type: &str,
    run_id: &str,
    execution_id: &str,
    expected_worktree_path: Option<&str>,
    emitter: Option<&super::stream_events::AgentEventEmitter>,
) -> bool {
    let mut summary = SUBAGENT_PANIC_SUMMARY.to_string();
    let mut retain_branch = false;
    let mut retain_worktree = subagent_type == "coder" && expected_worktree_path.is_some();
    let project_path = if subagent_type == "coder" {
        super::subagent_coder_project::for_child(child_session_id)
            .await
            .ok()
    } else {
        None
    };
    if let (Some(project), Some(worktree)) = (project_path.as_deref(), expected_worktree_path) {
        match super::subagent_task_change::capture(
            project,
            child_session_id,
            execution_id,
            std::path::Path::new(worktree),
        )
        .await
        {
            Ok(Some(metadata)) => {
                summary.push_str(&metadata);
                retain_branch = true;
                retain_worktree = false;
            }
            Ok(None) => retain_worktree = false,
            Err(_) => retain_branch = true,
        }
    }
    let completion = super::subagent_completion_events::persist_terminal(
        parent_session_id,
        child_session_id,
        subagent_type,
        super::subagent_status::FAILED,
        &summary,
        run_id,
        execution_id,
        false,
        emitter,
    )
    .await;
    cleanup(
        child_session_id,
        execution_id,
        expected_worktree_path,
        retain_worktree,
    )
    .await;
    if !retain_branch && !retain_worktree {
        if let Some(project) = project_path.as_deref() {
            super::subagent_task_change::delete_empty_workspace(
                project,
                child_session_id,
                execution_id,
            )
            .await;
        }
    }
    !matches!(completion, Ok(None))
}

async fn cleanup(
    child_session_id: &str,
    execution_id: &str,
    expected_worktree_path: Option<&str>,
    retain_worktree: bool,
) {
    if !retain_worktree {
        super::subagent_working_dir::cleanup_owned(
            child_session_id,
            execution_id,
            expected_worktree_path,
        )
        .await;
    }
    super::session_store::remove_session_lock(child_session_id).await;
}
