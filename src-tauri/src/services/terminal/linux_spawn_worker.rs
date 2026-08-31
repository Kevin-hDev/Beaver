use crate::services::work_registry::{ServiceWorkAdmission, ServiceWorkSupervisor};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;
use tokio::sync::oneshot;

type SpawnResult = Result<(u32, String), String>;
type SpawnOperation = Box<dyn FnOnce() -> SpawnResult + Send + 'static>;

enum SpawnRequest {
    Spawn(SpawnOperation, oneshot::Sender<SpawnResult>),
    #[cfg(test)]
    Probe(
        Box<dyn FnOnce() -> usize + Send + 'static>,
        oneshot::Sender<Result<ProbeResult, String>>,
    ),
    Shutdown,
}

struct WorkerState {
    sender: Option<SyncSender<SpawnRequest>>,
    join: Option<JoinHandle<()>>,
}

pub(super) struct LinuxSpawnWorker {
    work: ServiceWorkSupervisor<1>,
    state: Mutex<WorkerState>,
    closing: Arc<AtomicBool>,
}

#[cfg(test)]
#[derive(Debug)]
pub(super) struct ProbeResult {
    pub(super) thread_id: std::thread::ThreadId,
    pub(super) thread_name: Option<String>,
    pub(super) value: usize,
}

impl LinuxSpawnWorker {
    pub(super) fn new(work: ServiceWorkSupervisor<1>) -> Self {
        Self {
            work,
            state: Mutex::new(WorkerState {
                sender: None,
                join: None,
            }),
            closing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(super) async fn submit(&self, operation: SpawnOperation) -> SpawnResult {
        let sender = self.sender()?;
        let (completed, result) = oneshot::channel();
        sender
            .try_send(SpawnRequest::Spawn(operation, completed))
            .map_err(map_send_error)?;
        result.await.map_err(|_| terminal_error())?
    }

    pub(super) fn begin_closing(&self) {
        self.closing.store(true, Ordering::Release);
        self.work.begin_closing();
    }

    pub(super) async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.begin_closing();
        let (sender, join) = match self.state.lock() {
            Ok(mut state) => (state.sender.take(), state.join.take()),
            Err(_) => return false,
        };
        if let Some(sender) = sender {
            let _ = sender.try_send(SpawnRequest::Shutdown);
            drop(sender);
        }
        let joined = if let Some(join) = join {
            super::shutdown::run_until(deadline, move || {
                let _ = join.join();
            })
            .await
        } else {
            true
        };
        joined && self.work.stop_and_wait(deadline).await
    }

    fn sender(&self) -> Result<SyncSender<SpawnRequest>, String> {
        if self.closing.load(Ordering::Acquire) {
            return Err(shutting_down());
        }
        let mut state = self.state.lock().map_err(|_| terminal_error())?;
        if let Some(sender) = &state.sender {
            return Ok(sender.clone());
        }
        let admission = self.work.try_admit().map_err(|_| shutting_down())?;
        let (sender, receiver) = mpsc::sync_channel(16);
        let closing = Arc::clone(&self.closing);
        let join = std::thread::Builder::new()
            .name("beaver-terminal-linux-spawn".to_string())
            .spawn(move || worker_loop(receiver, admission, closing))
            .map_err(|_| terminal_error())?;
        state.sender = Some(sender.clone());
        state.join = Some(join);
        Ok(sender)
    }

    #[cfg(test)]
    pub(super) async fn run_test_probe(
        &self,
        operation: impl FnOnce() -> usize + Send + 'static,
    ) -> Result<ProbeResult, String> {
        let result = self.queue_test_probe(operation)?;
        result.await.map_err(|_| terminal_error())?
    }

    #[cfg(test)]
    pub(super) fn queue_test_probe(
        &self,
        operation: impl FnOnce() -> usize + Send + 'static,
    ) -> Result<oneshot::Receiver<Result<ProbeResult, String>>, String> {
        let sender = self.sender()?;
        let (completed, result) = oneshot::channel();
        sender
            .try_send(SpawnRequest::Probe(Box::new(operation), completed))
            .map_err(map_send_error)?;
        Ok(result)
    }

    #[cfg(test)]
    pub(super) fn diagnostics_for_test(
        &self,
    ) -> crate::services::work_registry::ServiceWorkDiagnostics {
        self.work.diagnostics()
    }
}

fn worker_loop(
    receiver: mpsc::Receiver<SpawnRequest>,
    _admission: ServiceWorkAdmission<1>,
    closing: Arc<AtomicBool>,
) {
    while let Ok(request) = receiver.recv() {
        match request {
            SpawnRequest::Spawn(operation, completed) => {
                let result = if closing.load(Ordering::Acquire) {
                    Err(shutting_down())
                } else {
                    catch_unwind(AssertUnwindSafe(operation))
                        .unwrap_or_else(|_| Err(terminal_error()))
                };
                let _ = completed.send(result);
            }
            #[cfg(test)]
            SpawnRequest::Probe(operation, completed) => {
                let result = if closing.load(Ordering::Acquire) {
                    Err(shutting_down())
                } else {
                    catch_unwind(AssertUnwindSafe(operation))
                        .map(|value| ProbeResult {
                            thread_id: std::thread::current().id(),
                            thread_name: std::thread::current().name().map(str::to_string),
                            value,
                        })
                        .map_err(|_| terminal_error())
                };
                let _ = completed.send(result);
            }
            SpawnRequest::Shutdown => break,
        }
    }
}

fn map_send_error(error: TrySendError<SpawnRequest>) -> String {
    match error {
        TrySendError::Full(_) => terminal_error(),
        TrySendError::Disconnected(_) => shutting_down(),
    }
}

fn terminal_error() -> String {
    "terminal-error".to_string()
}

fn shutting_down() -> String {
    "terminal-shutting-down".to_string()
}
