use super::error_codes;
use super::host_channel::{self, PendingRequests, SharedWriter};
use super::host_load_tracker::HostLoadTracker;
use super::host_paths::HostPaths;
use super::protocol::{HostExtensionSpec, RpcRequest};
use super::types::{HOST_REQUEST_TIMEOUT_MS, HOST_STOP_TIMEOUT_MS, MAX_PENDING_REQUESTS};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

const READER_STOP_TIMEOUT: Duration = Duration::from_secs(1);

pub struct HostProcess {
    child: Mutex<Child>,
    writer: SharedWriter,
    pending: PendingRequests,
    alive: Arc<AtomicBool>,
    load_tracker: Arc<HostLoadTracker>,
    reader_done: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
}

impl HostProcess {
    #[cfg(test)]
    pub(super) async fn hold_child_for_test(&self) -> tokio::sync::MutexGuard<'_, Child> {
        self.child.lock().await
    }

    pub async fn spawn(
        paths: &HostPaths,
        work: &super::work_supervision::ExtensionWorkServices,
    ) -> Result<Self, String> {
        // L'admission précède le spawn : pendant Closing, aucun enfant Node
        // transitoire ne doit franchir la frontière de fermeture.
        let reader_admission = work
            .try_admit_reader()
            .map_err(|error| error.public_code().to_string())?;
        let mut command = Command::new(&paths.node);
        command
            .arg(&paths.script)
            .current_dir(&paths.directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = crate::services::owned_process::OwnedProcess::spawn_tokio(
            &mut command,
            crate::services::process_tree::ProcessKind::ExtensionHost,
        )
        .await
        .map_err(|_| error_codes::HOST_UNAVAILABLE.to_string())?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| error_codes::HOST_UNAVAILABLE.to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| error_codes::HOST_UNAVAILABLE.to_string())?;
        let writer = Arc::new(Mutex::new(stdin));
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let alive = Arc::new(AtomicBool::new(true));
        let load_tracker = Arc::new(HostLoadTracker::default());
        let run_work = work.clone();
        let run_writer = writer.clone();
        let run_pending = pending.clone();
        let run_alive = alive.clone();
        let run_load_tracker = load_tracker.clone();
        let reader_finished =
            match reader_admission.spawn_with_completion(move |cancel| async move {
                super::host_reader::run(
                    stdout,
                    run_writer,
                    run_pending,
                    run_alive,
                    run_load_tracker,
                    run_work,
                    cancel,
                )
                .await;
            }) {
                Ok(completion) => completion,
                Err(_) => {
                    crate::services::process_tree::terminate_tokio(
                        &mut child,
                        crate::services::process_tree::ProcessKind::ExtensionHost,
                    )
                    .await;
                    return Err(error_codes::HOST_UNAVAILABLE.to_string());
                }
            };
        Ok(Self {
            child: Mutex::new(child),
            writer,
            pending,
            alive,
            load_tracker,
            reader_done: Mutex::new(Some(reader_finished)),
        })
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, String> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(error_codes::HOST_UNAVAILABLE.to_string());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            if pending.len() >= MAX_PENDING_REQUESTS {
                return Err(error_codes::HOST_BUSY.to_string());
            }
            pending.insert(id.clone(), sender);
        }
        if !self.alive.load(Ordering::Acquire) {
            self.pending.lock().await.remove(&id);
            return Err(error_codes::HOST_UNAVAILABLE.to_string());
        }
        let request = RpcRequest {
            jsonrpc: "2.0",
            id: &id,
            method,
            params,
        };
        if let Err(error) = host_channel::write(&self.writer, &request).await {
            self.pending.lock().await.remove(&id);
            return Err(error);
        }
        match tokio::time::timeout(
            Duration::from_millis(HOST_REQUEST_TIMEOUT_MS as u64),
            receiver,
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(error_codes::HOST_UNAVAILABLE.to_string()),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err(error_codes::HOST_TIMEOUT.to_string())
            }
        }
    }

    pub async fn load(&self, specification: &HostExtensionSpec) -> Result<Value, String> {
        self.load_tracker.arm(&specification.id).await?;
        let result = self
            .request("host.load", json!({"extension": specification}))
            .await;
        self.load_tracker.clear().await;
        result
    }

    pub async fn kill(&self, deadline: Instant) -> bool {
        self.alive.store(false, Ordering::Release);
        let Ok(mut child) =
            tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), self.child.lock())
                .await
        else {
            return false;
        };
        let terminated = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            crate::services::process_tree::terminate_tokio(
                &mut child,
                crate::services::process_tree::ProcessKind::ExtensionHost,
            ),
        )
        .await
        .is_ok();
        drop(child);
        let pending_failed = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            host_channel::fail_all(&self.pending),
        )
        .await
        .is_ok();
        terminated && pending_failed && wait_reader_done(&self.reader_done, deadline).await
    }
}

pub(super) async fn wait_reader_done(
    reader_done: &Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    deadline: Instant,
) -> bool {
    let reader_deadline = deadline.min(Instant::now() + READER_STOP_TIMEOUT);
    let Ok(mut slot) = tokio::time::timeout_at(
        tokio::time::Instant::from_std(reader_deadline),
        reader_done.lock(),
    )
    .await
    else {
        return false;
    };
    let Some(receiver) = slot.as_mut() else {
        return true;
    };
    if tokio::time::timeout_at(tokio::time::Instant::from_std(reader_deadline), receiver)
        .await
        .is_err()
    {
        return false;
    }
    slot.take();
    true
}

pub(super) fn stop_deadline() -> Instant {
    Instant::now() + Duration::from_millis(HOST_STOP_TIMEOUT_MS as u64)
}

#[cfg(test)]
#[path = "host_process_tests.rs"]
mod tests;
