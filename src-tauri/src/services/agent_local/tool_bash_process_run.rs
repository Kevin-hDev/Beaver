#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::tool_bash_changes::ChangeTracker;
use super::tool_bash_io::OutputEvent;
use super::tool_bash_output::ShellStream;
use super::tool_bash_session::{CompletionKind, ShellSession};
use super::tool_bash_storage::ShellOutputStore;

const PROGRESS_INTERVAL_MS: u64 = 250;
const FINAL_CHANGE_SETTLE_MS: u64 = 200;
const FINAL_GIT_CHANGE_SETTLE_MS: u64 = 25;
const KEEP_OUTPUT_AFTER_BYTES: usize = 28 * 1024;

pub(super) async fn run(
    session: &Arc<ShellSession>,
    child: &mut Child,
    stdout: ChildStdout,
    stderr: ChildStderr,
    mut store: ShellOutputStore,
    mut tracker: Option<ChangeTracker>,
    hard_timeout_secs: Option<u64>,
    agent_cancel: CancellationToken,
    shutdown: crate::services::work_registry::ServiceWorkCancellation,
    sandbox_cleanup: Option<std::path::PathBuf>,
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
    let mut shutdown_cancelled = false;

    let mut completion = loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                shutdown_cancelled = true;
                break CompletionKind::Cancelled;
            }
            _ = agent_cancel.cancelled() => break CompletionKind::Cancelled,
            _ = session_cancel.cancelled() => break CompletionKind::Cancelled,
            _ = session_stop.cancelled() => break CompletionKind::Stopped,
            _ = &mut timeout_wait => break CompletionKind::TimedOut,
            status = child.wait() => {
                break status.ok()
                    .map(|status| CompletionKind::Exited(status.code().unwrap_or(-1)))
                    .unwrap_or(CompletionKind::Failed);
            }
            event = receiver.recv(), if output_open => match event {
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
            },
            _ = tick.tick() => {
                refresh_changes(session, tracker.as_mut());
                session.emit_progress();
            }
        }
    };

    if !matches!(completion, CompletionKind::Exited(_)) {
        super::tool_bash_platform::terminate_process_tree(session.pid()).await;
        let terminated = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
        completion = super::tool_bash_completion::after_termination_attempt(
            completion,
            matches!(terminated, Ok(Ok(_))),
        );
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
    // La fermeture peut commencer après la sortie du processus mais avant le bilan final.
    shutdown_cancelled |= shutdown.is_cancelled();
    settle_changes(session, &mut tracker, shutdown_cancelled).await;
    session.emit_progress();
    session.close_stdin().await;

    let keep_output = session.total_output_bytes() > KEEP_OUTPUT_AFTER_BYTES;
    let output_path = store.finalize(keep_output).await.ok().flatten();
    if matches!(drain, super::tool_bash_io::DrainOutcome::Failed)
        || (output_path.is_none() && keep_output)
    {
        completion = CompletionKind::Failed;
    }
    let sandbox_warning = sandbox_cleanup
        .as_deref()
        .and_then(super::shell_sandbox_diagnostics::warning);
    session.set_sandbox_warning(sandbox_warning);
    session.complete(completion, output_path);
    super::shell_sandbox::cleanup_temp(sandbox_cleanup).await;
}

async fn settle_changes(
    session: &ShellSession,
    tracker: &mut Option<ChangeTracker>,
    shutdown_cancelled: bool,
) {
    let settle_ms = match (shutdown_cancelled, tracker.as_ref()) {
        (true, _) => 0,
        (false, Some(tracker)) if tracker.requires_event_settle() => FINAL_CHANGE_SETTLE_MS,
        (false, Some(_)) => FINAL_GIT_CHANGE_SETTLE_MS,
        (false, None) => 0,
    };
    if settle_ms > 0 {
        tokio::time::sleep(Duration::from_millis(settle_ms)).await;
    }
    let Some(tracker) = tracker.take() else {
        return;
    };
    let (changes, incomplete) = collect_final_changes(tracker, shutdown_cancelled, |tracker| {
        tracker.finish_changes()
    })
    .await;
    session.update_changes(changes, incomplete);
}

async fn collect_final_changes<Finish>(
    mut tracker: ChangeTracker,
    shutdown_cancelled: bool,
    finish: Finish,
) -> (Vec<super::types_tools::ToolFileChange>, bool)
where
    Finish: FnOnce(&mut ChangeTracker) -> (Vec<super::types_tools::ToolFileChange>, bool)
        + Send
        + 'static,
{
    if shutdown_cancelled {
        tracker.drain_ready();
        return (tracker.snapshot(false, None), true);
    }
    tokio::task::spawn_blocking(move || finish(&mut tracker))
        .await
        .unwrap_or_else(|_| (Vec::new(), true))
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

#[cfg(test)]
#[path = "tool_bash_process_run_tests.rs"]
mod tests;
