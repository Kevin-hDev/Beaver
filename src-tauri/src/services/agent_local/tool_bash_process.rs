use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::tool_bash_changes::ChangeTracker;
use super::tool_bash_io::OutputEvent;
use super::tool_bash_progress::ShellProgress;
use super::tool_bash_session::{CompletionKind, ShellSession};
use super::tool_bash_storage::ShellOutputStore;

const PROGRESS_INTERVAL_MS: u64 = 250;
const FINAL_CHANGE_SETTLE_MS: u64 = 200;
const KEEP_OUTPUT_AFTER_BYTES: usize = 28 * 1024;

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
    let mut command = command?;
    let tracker = tracker.ok();
    if request.agent_cancel.is_cancelled() {
        return Err("Commande annulee.".to_string());
    }
    let store = ShellOutputStore::prepare(request.owner_session_id)?;
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => {
            let _ = store.finalize(false).await;
            return Err("Lancement du shell impossible.".to_string());
        }
    };
    let Some(pid) = child.id() else {
        let _ = child.kill().await;
        let _ = store.finalize(false).await;
        return Err("Lancement du shell impossible.".to_string());
    };
    let pipes = (child.stdin.take(), child.stdout.take(), child.stderr.take());
    let (Some(stdin), Some(stdout), Some(stderr)) = pipes else {
        super::tool_bash_platform::terminate_process_tree(pid).await;
        let _ = child.wait().await;
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
    if let Err(error) = super::tool_bash_registry::insert(Arc::clone(&session)) {
        super::tool_bash_platform::terminate_process_tree(pid).await;
        let _ = child.wait().await;
        let _ = store.finalize(false).await;
        return Err(error);
    }

    let task_session = Arc::clone(&session);
    tokio::spawn(async move {
        run_process(
            &task_session,
            &mut child,
            stdout,
            stderr,
            store,
            tracker,
            request.hard_timeout_secs,
            request.agent_cancel,
        )
        .await;
    });
    Ok(session)
}

async fn run_process(
    session: &Arc<ShellSession>,
    child: &mut Child,
    stdout: ChildStdout,
    stderr: ChildStderr,
    mut store: ShellOutputStore,
    mut tracker: Option<ChangeTracker>,
    hard_timeout_secs: Option<u64>,
    agent_cancel: CancellationToken,
) {
    let (sender, mut receiver) = mpsc::channel(super::tool_bash_io::OUTPUT_CHANNEL_SIZE);
    let readers = [
        super::tool_bash_io::spawn_reader(stdout, sender.clone()),
        super::tool_bash_io::spawn_reader(stderr, sender),
    ];
    let session_cancel = session.cancellation();
    let timeout_wait = wait_for_timeout(hard_timeout_secs);
    tokio::pin!(timeout_wait);
    let mut tick = tokio::time::interval(Duration::from_millis(PROGRESS_INTERVAL_MS));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let completion = loop {
        tokio::select! {
            _ = agent_cancel.cancelled() => break CompletionKind::Cancelled,
            _ = session_cancel.cancelled() => break CompletionKind::Cancelled,
            _ = &mut timeout_wait => break CompletionKind::TimedOut,
            status = child.wait() => {
                break status
                    .ok()
                    .map(|status| CompletionKind::Exited(status.code().unwrap_or(-1)))
                    .unwrap_or(CompletionKind::Failed);
            }
            event = receiver.recv() => {
                match event {
                    Some(OutputEvent::Data(mut bytes)) => {
                        use zeroize::Zeroize;
                        if store.append(&bytes).await.is_err() {
                            bytes.zeroize();
                            break CompletionKind::Failed;
                        }
                        session.append_output(&bytes);
                        bytes.zeroize();
                    }
                    Some(OutputEvent::Failed) => break CompletionKind::Failed,
                    None => {}
                }
            }
            _ = tick.tick() => {
                refresh_changes(session, tracker.as_mut());
                session.emit_progress();
            }
        }
    };

    if !matches!(completion, CompletionKind::Exited(_)) {
        super::tool_bash_platform::terminate_process_tree(session.pid()).await;
        let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
    }
    let fully_drained = super::tool_bash_io::drain(session, &mut store, &mut receiver).await;
    if !fully_drained && matches!(completion, CompletionKind::Exited(_)) {
        super::tool_bash_platform::terminate_process_tree(session.pid()).await;
    }
    for reader in readers {
        reader.abort();
        let _ = reader.await;
    }
    super::tool_bash_io::clear_pending(&mut receiver);
    tokio::time::sleep(Duration::from_millis(FINAL_CHANGE_SETTLE_MS)).await;
    refresh_changes(session, tracker.as_mut());
    session.emit_progress();
    session.close_stdin().await;

    let keep_output = session.total_output_bytes() > KEEP_OUTPUT_AFTER_BYTES;
    let output_path = store.finalize(keep_output).await.ok().flatten();
    let completion = if output_path.is_none() && keep_output {
        CompletionKind::Failed
    } else {
        completion
    };
    session.complete(completion, output_path);
}

fn refresh_changes(session: &ShellSession, tracker: Option<&mut ChangeTracker>) {
    if let Some(tracker) = tracker {
        if let Some((changes, incomplete)) = tracker.updated_changes() {
            session.update_changes(changes, incomplete);
        }
    }
}

async fn wait_for_timeout(timeout_secs: Option<u64>) {
    match timeout_secs {
        Some(seconds) => tokio::time::sleep(Duration::from_secs(seconds)).await,
        None => std::future::pending::<()>().await,
    }
}
