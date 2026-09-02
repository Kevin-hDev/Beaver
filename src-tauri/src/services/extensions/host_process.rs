use super::error_codes;
use super::host_channel::{self, PendingRequests, SharedWriter};
use super::host_load_tracker::HostLoadTracker;
use super::protocol::{HostExtensionSpec, RpcRequest};
use super::types::{HOST_REQUEST_TIMEOUT_MS, HOST_STOP_TIMEOUT_MS, MAX_PENDING_REQUESTS};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Child;
use tokio::sync::Mutex;

const READER_STOP_TIMEOUT: Duration = Duration::from_secs(1);

pub struct HostProcess {
    child: Mutex<Child>,
    writer: SharedWriter,
    pending: PendingRequests,
    alive: Arc<AtomicBool>,
    channel_cancel: tokio_util::sync::CancellationToken,
    load_tracker: Arc<HostLoadTracker>,
    reader_done: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    process_scope: crate::services::owned_process::OwnedProcessScope,
    _test_temporary_directory: Option<tempfile::TempDir>,
}

impl HostProcess {
    pub(super) fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(super) async fn hold_child_for_test(&self) -> tokio::sync::MutexGuard<'_, Child> {
        self.child.lock().await
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, String> {
        if !self.is_alive() {
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
        if !self.is_alive() {
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
        self.channel_cancel.cancel();
        let Ok(mut child) =
            tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), self.child.lock())
                .await
        else {
            return false;
        };
        let terminated = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            crate::services::process_tree::terminate_tokio_scoped(
                &mut child,
                crate::services::process_tree::ProcessKind::ExtensionHost,
                &self.process_scope,
            ),
        )
        .await
        .is_ok_and(|scope_terminated| scope_terminated);
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

#[path = "host_process_spawn.rs"]
mod spawn;

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
