use super::host_channel::{self, PendingRequests, SharedWriter, MAX_PENDING_REQUESTS};
use super::host_paths::HostPaths;
use super::protocol::RpcRequest;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const KILL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct HostProcess {
    child: Mutex<Child>,
    writer: SharedWriter,
    pending: PendingRequests,
    alive: Arc<AtomicBool>,
    reader: Mutex<Option<JoinHandle<()>>>,
}

impl HostProcess {
    pub fn spawn(paths: &HostPaths) -> Result<Self, String> {
        let mut command = Command::new(&paths.node);
        command
            .arg(&paths.script)
            .current_dir(&paths.directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|_| "Impossible de démarrer l'hôte d'extensions.".to_string())?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Hôte d'extensions indisponible.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Hôte d'extensions indisponible.".to_string())?;
        let writer = Arc::new(Mutex::new(stdin));
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let alive = Arc::new(AtomicBool::new(true));
        let reader = tokio::spawn(super::host_reader::run(
            stdout,
            writer.clone(),
            pending.clone(),
            alive.clone(),
        ));
        Ok(Self {
            child: Mutex::new(child),
            writer,
            pending,
            alive,
            reader: Mutex::new(Some(reader)),
        })
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, String> {
        if !self.alive.load(Ordering::Acquire) {
            return Err("Hôte d'extensions indisponible.".to_string());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            if pending.len() >= MAX_PENDING_REQUESTS {
                return Err("Trop de requêtes vers l'hôte d'extensions.".to_string());
            }
            pending.insert(id.clone(), sender);
        }
        if !self.alive.load(Ordering::Acquire) {
            self.pending.lock().await.remove(&id);
            return Err("Hôte d'extensions indisponible.".to_string());
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
            Ok(Err(_)) => Err("Hôte d'extensions indisponible.".to_string()),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err("L'hôte d'extensions ne répond pas.".to_string())
            }
        }
    }

    pub async fn kill(&self) {
        self.alive.store(false, Ordering::Release);
        let mut child = self.child.lock().await;
        let _ = child.start_kill();
        let _ = tokio::time::timeout(KILL_TIMEOUT, child.wait()).await;
        drop(child);
        if let Some(reader) = self.reader.lock().await.take() {
            reader.abort();
        }
        host_channel::fail_all(&self.pending).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn matches_concurrent_out_of_order_responses_by_id() {
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("host.mjs");
        std::fs::write(
            &script,
            r#"import readline from "node:readline";
const lines = readline.createInterface({ input: process.stdin });
lines.on("line", (line) => {
  const message = JSON.parse(line);
  setTimeout(() => process.stdout.write(JSON.stringify({
    jsonrpc: "2.0", id: message.id, result: message.params.value
  }) + "\n"), message.params.delay);
});"#,
        )
        .unwrap();
        let paths = HostPaths {
            node: which::which("node").unwrap(),
            script,
            directory: directory.path().to_path_buf(),
        };
        let host = Arc::new(HostProcess::spawn(&paths).unwrap());
        let slow_host = host.clone();
        let fast_host = host.clone();
        let (slow, fast) = tokio::join!(
            slow_host.request("test", json!({"value": "slow", "delay": 50})),
            fast_host.request("test", json!({"value": "fast", "delay": 1})),
        );

        assert_eq!(slow.unwrap(), json!("slow"));
        assert_eq!(fast.unwrap(), json!("fast"));
        host.kill().await;
        assert!(host.request("test", json!({})).await.is_err());
    }
}
