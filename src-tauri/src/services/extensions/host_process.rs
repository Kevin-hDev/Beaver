use super::error_codes;
use super::host_channel::{self, PendingRequests, SharedWriter};
use super::host_load_tracker::HostLoadTracker;
use super::protocol::{HostExtensionSpec, RpcRequest};
use super::types::HOST_REQUEST_TIMEOUT_MS;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Child;
use tokio::sync::Mutex;

pub struct HostProcess {
    child: Mutex<Child>,
    writer: SharedWriter,
    pending: PendingRequests,
    alive: Arc<AtomicBool>,
    reader_cancel: tokio_util::sync::CancellationToken,
    load_tracker: Arc<HostLoadTracker>,
    reader_done: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    process_scope: crate::services::owned_process::OwnedProcessScope,
    root_pid: u32,
    _test_temporary_directory: Option<tempfile::TempDir>,
}

struct PendingCleanup {
    pending: PendingRequests,
    id: String,
}

impl Drop for PendingCleanup {
    fn drop(&mut self) {
        let _ = host_channel::remove(&self.pending, &self.id);
    }
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
        host_channel::insert(&self.pending, id.clone(), sender)?;
        let _cleanup = PendingCleanup {
            pending: Arc::clone(&self.pending),
            id: id.clone(),
        };
        if !self.is_alive() {
            return Err(error_codes::HOST_UNAVAILABLE.to_string());
        }
        let request = RpcRequest {
            jsonrpc: "2.0",
            id: &id,
            method,
            params,
        };
        host_channel::write(&self.writer, &request).await?;
        match tokio::time::timeout(
            Duration::from_millis(HOST_REQUEST_TIMEOUT_MS as u64),
            receiver,
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(error_codes::HOST_UNAVAILABLE.to_string()),
            Err(_) => Err(error_codes::HOST_TIMEOUT.to_string()),
        }
    }

    pub async fn load(
        &self,
        specification: &HostExtensionSpec,
        attempts: u8,
    ) -> Result<Value, String> {
        self.load_tracker.arm(&specification.id).await?;
        if let Err(error) = super::loading_marker::start(&specification.id, attempts) {
            self.load_tracker.clear().await;
            return Err(error);
        }
        let result = self
            .request("host.load", json!({"extension": specification}))
            .await;
        self.load_tracker.clear().await;
        result
    }

    pub async fn kill(&self, deadline: Instant) -> bool {
        self.alive.store(false, Ordering::Release);
        self.reader_cancel.cancel();
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
                self.root_pid,
                deadline,
            ),
        )
        .await
        .is_ok_and(|scope_terminated| scope_terminated);
        drop(child);
        host_channel::fail_all(&self.pending);
        terminated && wait_reader_done(&self.reader_done, deadline).await
    }

    #[cfg(test)]
    pub(super) fn pending_len_for_test(&self) -> usize {
        host_channel::len(&self.pending)
    }
}

#[path = "host_process_spawn.rs"]
mod spawn;
pub(super) use spawn::HostSpawnBinding;

pub(super) async fn wait_reader_done(
    reader_done: &Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    deadline: Instant,
) -> bool {
    let Ok(mut slot) =
        tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), reader_done.lock()).await
    else {
        return false;
    };
    let Some(receiver) = slot.as_mut() else {
        return true;
    };
    if tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), receiver)
        .await
        .is_err()
    {
        return false;
    }
    slot.take();
    true
}

#[cfg(test)]
#[path = "host_process_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "host_process_prepared_tests.rs"]
mod prepared_tests;

#[cfg(test)]
fn test_extension_work() -> super::work_supervision::ExtensionWorkServices {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor())
}
