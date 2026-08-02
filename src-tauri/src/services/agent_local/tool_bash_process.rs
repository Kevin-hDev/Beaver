use std::path::Path;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use super::tool_bash_changes::ChangeTracker;
use super::tool_bash_progress::ShellProgress;
use super::tool_bash_session::ShellSession;
use super::tool_bash_storage::ShellOutputStore;

pub struct SpawnRequest<'a> {
    pub command: &'a str,
    pub working_dir: &'a Path,
    pub owner_session_id: &'a str,
    pub hard_timeout_secs: Option<u64>,
    pub progress: Option<ShellProgress>,
    pub agent_cancel: CancellationToken,
}

pub async fn spawn(request: SpawnRequest<'_>) -> Result<Arc<ShellSession>, String> {
    if request.agent_cancel.is_cancelled() {
        return Err("Commande annulee.".to_string());
    }
    let process_id = uuid::Uuid::new_v4().to_string();
    let (command, tracker) = tokio::join!(
        super::tool_bash_shell::build_command(
            request.command,
            request.working_dir,
            request.owner_session_id,
        ),
        ChangeTracker::start(request.working_dir),
    );
    let built = command?;
    let mut command = built.command;
    let sandbox_cleanup = built.cleanup_dir;
    let tracker = tracker.ok();
    let tracking_unavailable = tracker.is_none();
    if request.agent_cancel.is_cancelled() {
        super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
        return Err("Commande annulee.".to_string());
    }
    let store = match ShellOutputStore::prepare(request.owner_session_id) {
        Ok(store) => store,
        Err(error) => {
            super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
            return Err(error);
        }
    };
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
            let _ = store.finalize(false).await;
            return Err(super::tool_bash_spawn_error::message(error));
        }
    };
    let Some(pid) = child.id() else {
        let _ = child.kill().await;
        super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
        let _ = store.finalize(false).await;
        return Err("Lancement du shell impossible.".to_string());
    };
    let pipes = (child.stdin.take(), child.stdout.take(), child.stderr.take());
    let (Some(stdin), Some(stdout), Some(stderr)) = pipes else {
        super::tool_bash_platform::terminate_process_tree(pid).await;
        let _ = child.wait().await;
        super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
        let _ = store.finalize(false).await;
        return Err("Lancement du shell impossible.".to_string());
    };
    let session = ShellSession::new(
        process_id,
        request.owner_session_id.to_string(),
        pid,
        stdin,
        store.relative_path().to_string(),
        request.progress,
    );
    if tracking_unavailable {
        session.update_changes(Vec::new(), true);
    }
    if let Err(error) =
        super::tool_bash_registry::insert(Arc::clone(&session), request.command)
    {
        super::tool_bash_platform::terminate_process_tree(pid).await;
        let _ = child.wait().await;
        super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
        let _ = store.finalize(false).await;
        return Err(error);
    }

    let task_session = Arc::clone(&session);
    tokio::spawn(async move {
        super::tool_bash_process_run::run(
            &task_session,
            &mut child,
            stdout,
            stderr,
            store,
            tracker,
            request.hard_timeout_secs,
            request.agent_cancel,
            sandbox_cleanup,
        )
        .await;
    });
    Ok(session)
}
