use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::tool_bash_changes::ChangeTracker;
use super::tool_bash_io::OutputEvent;
use super::tool_bash_output::ShellStream;
use super::tool_bash_progress::ShellProgress;
use super::tool_bash_session::{CompletionKind, ShellSession};
use super::tool_bash_storage::ShellOutputStore;

const PROGRESS_INTERVAL_MS: u64 = 250;
const FINAL_CHANGE_SETTLE_MS: u64 = 200;
const FINAL_GIT_CHANGE_SETTLE_MS: u64 = 25;
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
    let tracking_unavailable = tracker.is_none();
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
    if tracking_unavailable {
        session.update_changes(Vec::new(), true);
    }
    if let Err(error) =
        super::tool_bash_registry::insert(Arc::clone(&session), request.command)
    {
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
        super::tool_bash_io::spawn_reader(stdout, ShellStream::Stdout, sender.clone()),
        super::tool_bash_io::spawn_reader(stderr, ShellStream::Stderr, sender),
    ];
    let session_stop = session.stop_token();
    let session_cancel = session.cancellation();
    let timeout_wait = wait_for_timeout(hard_timeout_secs);
    tokio::pin!(timeout_wait);
    let mut tick = tokio::time::interval(Duration::from_millis(PROGRESS_INTERVAL_MS));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut output_open = true;

    let mut completion = loop {
        tokio::select! {
            _ = agent_cancel.cancelled() => break CompletionKind::Cancelled,
            _ = session_cancel.cancelled() => break CompletionKind::Cancelled,
            _ = session_stop.cancelled() => break CompletionKind::Stopped,
            _ = &mut timeout_wait => break CompletionKind::TimedOut,
            status = child.wait() => {
                break status
                    .ok()
                    .map(|status| CompletionKind::Exited(status.code().unwrap_or(-1)))
                    .unwrap_or(CompletionKind::Failed);
            }
            event = receiver.recv(), if output_open => {
                match event {
                    Some(OutputEvent::Data(stream, mut bytes)) => {
                        use zeroize::Zeroize;
                        if store.append(&bytes).await.is_err() {
                            bytes.zeroize();
                            break CompletionKind::Failed;
                        }
                        session.append_output(stream, &bytes);
                        bytes.zeroize();
                    }
                    Some(OutputEvent::Failed) => break CompletionKind::Failed,
                    None => output_open = false,
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
        let terminated = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
        if !matches!(terminated, Ok(Ok(_))) {
            completion = CompletionKind::Failed;
        }
    }
    let drain = super::tool_bash_io::drain(session, &mut store, &mut receiver).await;
    if matches!(drain, super::tool_bash_io::DrainOutcome::TimedOut) {
        session.mark_output_incomplete();
    }
    for reader in readers {
        reader.abort();
        let _ = reader.await;
    }
    super::tool_bash_io::clear_pending(&mut receiver);
    let settle_ms = match tracker.as_ref() {
        Some(tracker) if tracker.requires_event_settle() => FINAL_CHANGE_SETTLE_MS,
        Some(_) => FINAL_GIT_CHANGE_SETTLE_MS,
        None => 0,
    };
    if settle_ms > 0 {
        tokio::time::sleep(Duration::from_millis(settle_ms)).await;
    }
    finish_changes(session, &mut tracker).await;
    session.emit_progress();
    session.close_stdin().await;

    let keep_output = session.total_output_bytes() > KEEP_OUTPUT_AFTER_BYTES;
    let output_path = store.finalize(keep_output).await.ok().flatten();
    let completion = if matches!(drain, super::tool_bash_io::DrainOutcome::Failed)
        || (output_path.is_none() && keep_output)
    {
        CompletionKind::Failed
    } else {
        completion
    };
    session.complete(completion, output_path);
}

async fn finish_changes(session: &ShellSession, tracker: &mut Option<ChangeTracker>) {
    let Some(mut tracker) = tracker.take() else {
        return;
    };
    match tokio::task::spawn_blocking(move || tracker.finish_changes()).await {
        Ok((changes, incomplete)) => session.update_changes(changes, incomplete),
        Err(_) => session.update_changes(Vec::new(), true),
    }
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
