use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use tokio::process::ChildStdin;
use tokio::sync::{Mutex as AsyncMutex, Notify};
use tokio_util::sync::CancellationToken;

use super::tool_bash_output::{ShellOutputBuffer, ShellStream};
use super::tool_bash_progress::ShellProgress;
use super::types_tools::ToolFileChange;

#[derive(Clone, Copy)]
pub enum CompletionKind {
    Exited(i32),
    Cancelled,
    TimedOut,
    Failed,
}

pub struct ShellSessionSnapshot {
    pub stdout: String,
    pub stderr: String,
    pub running: bool,
    pub completion: Option<CompletionKind>,
    pub elapsed_ms: u64,
    pub output_path: Option<String>,
    pub output_truncated: bool,
    pub changes: Vec<ToolFileChange>,
    pub tracking_incomplete: bool,
}

pub struct ShellSession {
    id: String,
    owner_session_id: String,
    pid: u32,
    started: Instant,
    state: Mutex<SessionState>,
    stdin: AsyncMutex<Option<ChildStdin>>,
    notify: Notify,
    cancel: CancellationToken,
}

struct SessionState {
    output: ShellOutputBuffer,
    completion: Option<CompletionKind>,
    output_path: Option<String>,
    changes: Vec<ToolFileChange>,
    tracking_incomplete: bool,
    progress: Option<ShellProgress>,
    last_progress_bytes: usize,
    last_progress_elapsed_ms: u64,
}

impl ShellSession {
    pub fn new(
        id: String,
        owner_session_id: String,
        pid: u32,
        stdin: ChildStdin,
        output_path: String,
        progress: Option<ShellProgress>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            owner_session_id,
            pid,
            started: Instant::now(),
            state: Mutex::new(SessionState {
                output: ShellOutputBuffer::default(),
                completion: None,
                output_path: Some(output_path),
                changes: Vec::new(),
                tracking_incomplete: false,
                progress,
                last_progress_bytes: 0,
                last_progress_elapsed_ms: 0,
            }),
            stdin: AsyncMutex::new(Some(stdin)),
            notify: Notify::new(),
            cancel: CancellationToken::new(),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn owner_session_id(&self) -> &str {
        &self.owner_session_id
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn cancellation(&self) -> CancellationToken {
        self.cancel.clone()
    }

    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    pub fn is_done(&self) -> bool {
        self.lock_state().completion.is_some()
    }

    pub fn append_output(&self, stream: ShellStream, bytes: &[u8]) {
        self.lock_state().output.append(stream, bytes);
    }

    pub fn update_changes(&self, changes: Vec<ToolFileChange>, incomplete: bool) {
        let mut state = self.lock_state();
        state.changes = changes;
        state.tracking_incomplete = incomplete;
    }

    pub fn complete(&self, completion: CompletionKind, output_path: Option<String>) {
        let mut state = self.lock_state();
        state.completion = Some(completion);
        state.output_path = output_path;
        state.progress = None;
        drop(state);
        self.notify.notify_waiters();
        self.notify.notify_one();
    }

    pub fn snapshot(&self) -> ShellSessionSnapshot {
        let mut state = self.lock_state();
        let pending = state.output.take_pending();
        ShellSessionSnapshot {
            stdout: pending.stdout,
            stderr: pending.stderr,
            running: state.completion.is_none(),
            completion: state.completion,
            elapsed_ms: self.started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            output_path: state.output_path.clone(),
            output_truncated: pending.truncated,
            changes: state.changes.clone(),
            tracking_incomplete: state.tracking_incomplete,
        }
    }

    pub fn total_output_bytes(&self) -> usize {
        self.lock_state().output.total_bytes()
    }

    pub fn set_progress(&self, progress: Option<ShellProgress>) {
        self.lock_state().progress = progress;
    }

    pub fn emit_progress(&self) {
        let elapsed_ms = self.started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        let (progress, preview, elapsed_ms) = {
            let mut state = self.lock_state();
            let total = state.output.total_bytes();
            let quiet_for = elapsed_ms.saturating_sub(state.last_progress_elapsed_ms);
            if total == state.last_progress_bytes && quiet_for < 1_000 {
                return;
            }
            state.last_progress_bytes = total;
            state.last_progress_elapsed_ms = elapsed_ms;
            (
                state.progress.clone(),
                state.output.live_preview(),
                elapsed_ms,
            )
        };
        if let Some(progress) = progress {
            progress.emit(&preview, elapsed_ms);
        }
    }

    pub async fn write_input(&self, input: &[u8]) -> Result<(), String> {
        use tokio::io::AsyncWriteExt;
        let mut stdin = self.stdin.lock().await;
        let Some(stdin) = stdin.as_mut() else {
            return Err("Processus shell termine.".to_string());
        };
        stdin
            .write_all(input)
            .await
            .map_err(|_| "Ecriture vers le shell impossible.".to_string())
    }

    pub async fn close_stdin(&self) {
        self.stdin.lock().await.take();
    }

    pub async fn notified(&self) {
        self.notify.notified().await;
    }

    fn lock_state(&self) -> MutexGuard<'_, SessionState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
}
