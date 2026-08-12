use super::error_codes;
use super::host_channel::{self, PendingRequests, SharedWriter};
use super::host_paths::HostPaths;
use super::protocol::RpcRequest;
use super::types::MAX_PENDING_REQUESTS;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const READER_STOP_TIMEOUT: Duration = Duration::from_secs(1);

pub struct HostProcess {
    child: Mutex<Child>,
    writer: SharedWriter,
    pending: PendingRequests,
    alive: Arc<AtomicBool>,
    reader_done: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
}

impl HostProcess {
    pub async fn spawn(
        paths: &HostPaths,
        work: &super::work_supervision::ExtensionWorkServices,
    ) -> Result<Self, String> {
        let mut command = Command::new(&paths.node);
        command
            .arg(&paths.script)
            .current_dir(&paths.directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        crate::services::process_tree::configure_tokio(&mut command);
        let mut child = command
            .spawn()
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
        let reader_work = work.clone();
        let run_work = work.clone();
        let run_writer = writer.clone();
        let run_pending = pending.clone();
        let run_alive = alive.clone();
        let (reader_done, reader_finished) = tokio::sync::oneshot::channel();
        if reader_work
            .spawn_reader(move |cancel| async move {
                super::host_reader::run(
                    stdout,
                    run_writer,
                    run_pending,
                    run_alive,
                    run_work,
                    cancel,
                )
                .await;
                let _ = reader_done.send(());
            })
            .is_err()
        {
            crate::services::process_tree::terminate_tokio(
                &mut child,
                crate::services::process_tree::ProcessKind::ExtensionHost,
            )
            .await;
            return Err(error_codes::HOST_UNAVAILABLE.to_string());
        }
        Ok(Self {
            child: Mutex::new(child),
            writer,
            pending,
            alive,
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
        match tokio::time::timeout(REQUEST_TIMEOUT, receiver).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(error_codes::HOST_UNAVAILABLE.to_string()),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err(error_codes::HOST_TIMEOUT.to_string())
            }
        }
    }

    pub async fn kill(&self) {
        self.alive.store(false, Ordering::Release);
        let mut child = self.child.lock().await;
        crate::services::process_tree::terminate_tokio(
            &mut child,
            crate::services::process_tree::ProcessKind::ExtensionHost,
        )
        .await;
        drop(child);
        host_channel::fail_all(&self.pending).await;
        if let Some(reader_done) = self.reader_done.lock().await.take() {
            let _ = tokio::time::timeout(READER_STOP_TIMEOUT, reader_done).await;
        }
    }
}

#[cfg(test)]
#[path = "host_process_tests.rs"]
mod tests;
