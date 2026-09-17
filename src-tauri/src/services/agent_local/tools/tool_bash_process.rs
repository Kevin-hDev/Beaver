use crate::services::work_registry::{ServiceWorkAdmission, ServiceWorkCancellation};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

use super::agent_work_supervision::MAX_ACTIVE_SHELLS;
use super::tool_bash_changes::ChangeTracker;
use super::tool_bash_progress::ShellProgress;
use super::tool_bash_session::ShellSession;
use super::tool_bash_storage::ShellOutputStore;

pub struct SpawnRequest {
    pub command: Zeroizing<String>,
    pub working_dir: PathBuf,
    pub owner_session_id: String,
    pub hard_timeout_secs: Option<u64>,
    pub progress: Option<ShellProgress>,
    pub agent_cancel: CancellationToken,
}

struct PreparedProcess {
    session: Arc<ShellSession>,
    child: Child,
    stdout: ChildStdout,
    stderr: ChildStderr,
    store: ShellOutputStore,
    tracker: Option<ChangeTracker>,
    sandbox_cleanup: Option<PathBuf>,
}

pub async fn spawn(
    request: SpawnRequest,
    admission: ServiceWorkAdmission<MAX_ACTIVE_SHELLS>,
) -> Result<Arc<ShellSession>, String> {
    let (started_tx, started_rx) = oneshot::channel();
    admission
        .spawn(move |shutdown| run_owned(request, shutdown, started_tx))
        .map_err(|error| error.public_code().to_string())?;
    started_rx
        .await
        .map_err(|_| "Lancement du shell impossible.".to_string())?
}

async fn run_owned(
    request: SpawnRequest,
    shutdown: ServiceWorkCancellation,
    started: oneshot::Sender<Result<Arc<ShellSession>, String>>,
) {
    let mut process = match prepare(&request, &shutdown).await {
        Ok(process) => process,
        Err(error) => {
            let _ = started.send(Err(error));
            return;
        }
    };
    let task_session = Arc::clone(&process.session);
    if started.send(Ok(Arc::clone(&process.session))).is_err() {
        process.session.cancel();
    }
    super::tool_bash_process_run::run(
        &task_session,
        &mut process.child,
        process.stdout,
        process.stderr,
        process.store,
        process.tracker,
        request.hard_timeout_secs,
        request.agent_cancel,
        shutdown,
        process.sandbox_cleanup,
    )
    .await;
}

async fn prepare(
    request: &SpawnRequest,
    shutdown: &ServiceWorkCancellation,
) -> Result<PreparedProcess, String> {
    if request.agent_cancel.is_cancelled() || shutdown.is_cancelled() {
        return Err("Commande annulee.".to_string());
    }
    let process_id = uuid::Uuid::new_v4().to_string();
    let (command, tracker) = tokio::join!(
        super::tool_bash_shell::build_command(
            request.command.as_str(),
            &request.working_dir,
            &request.owner_session_id,
        ),
        ChangeTracker::start(&request.working_dir),
    );
    let built = command?;
    let mut command = built.command;
    let sandbox_cleanup = built.cleanup_dir;
    let tracker = tracker.ok();
    let tracking_unavailable = tracker.is_none();
    if request.agent_cancel.is_cancelled() || shutdown.is_cancelled() {
        super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
        return Err("Commande annulee.".to_string());
    }
    let store = match ShellOutputStore::prepare(&request.owner_session_id) {
        Ok(store) => store,
        Err(error) => {
            super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
            return Err(error);
        }
    };
    let mut child = match crate::services::owned_process::OwnedProcess::spawn_tokio(
        &mut command,
        crate::services::process_tree::ProcessKind::AgentShell,
    )
    .await
    {
        Ok(child) => child,
        Err(crate::services::owned_process::OwnedProcessError::Spawn(kind)) => {
            super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
            let _ = store.finalize(false).await;
            return Err(super::tool_bash_spawn_error::message(kind));
        }
        Err(crate::services::owned_process::OwnedProcessError::Admission) => {
            super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
            let _ = store.finalize(false).await;
            return Err("Lancement du shell impossible.".to_string());
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
        request.owner_session_id.clone(),
        pid,
        stdin,
        store.relative_path().to_string(),
        request.progress.clone(),
    );
    if tracking_unavailable {
        session.update_changes(Vec::new(), true);
    }
    if let Err(error) =
        super::tool_bash_registry::insert(Arc::clone(&session), request.command.as_str())
    {
        super::tool_bash_platform::terminate_process_tree(pid).await;
        let _ = child.wait().await;
        super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
        let _ = store.finalize(false).await;
        return Err(error);
    }
    Ok(PreparedProcess {
        session,
        child,
        stdout,
        stderr,
        store,
        tracker,
        sandbox_cleanup,
    })
}
