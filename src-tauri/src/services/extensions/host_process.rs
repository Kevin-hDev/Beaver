use super::core_bridge::CoreResponse;
use super::host_paths::HostPaths;
use super::protocol::{RpcError, RpcErrorBody, RpcRequest, RpcResult};
use super::types::MAX_MESSAGE_BYTES;
use serde::Serialize;
use serde_json::Value;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use zeroize::Zeroizing;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

pub struct HostProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
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
        let (key, value) = super::host_paths::runtime_env()?;
        command.env(key, value);
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
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let request = RpcRequest {
            jsonrpc: "2.0",
            id: &id,
            method,
            params,
        };
        self.write(&request).await?;
        tokio::time::timeout(REQUEST_TIMEOUT, self.wait_for_response(&id))
            .await
            .map_err(|_| "L'hôte d'extensions ne répond pas.".to_string())?
    }

    pub async fn kill(mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }

    async fn wait_for_response(&mut self, expected_id: &str) -> Result<Value, String> {
        loop {
            let bytes = read_bounded_line(&mut self.stdout).await?;
            let message: Value = serde_json::from_slice(&bytes)
                .map_err(|_| "Réponse de l'hôte d'extensions invalide.".to_string())?;
            super::validation::message(&message)?;
            let object = super::protocol::envelope(&message)?;
            let id = object
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
            if let Some(method) = object.get("method").and_then(Value::as_str) {
                self.handle_core_request(id, method, object.get("params"))
                    .await?;
                continue;
            }
            if id != expected_id {
                return Err("Réponse de l'hôte d'extensions invalide.".to_string());
            }
            if object.contains_key("error") {
                return Err("L'hôte d'extensions a refusé la requête.".to_string());
            }
            return object
                .get("result")
                .cloned()
                .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string());
        }
    }

    async fn handle_core_request(
        &mut self,
        id: &str,
        method: &str,
        params: Option<&Value>,
    ) -> Result<(), String> {
        match super::core_bridge::call(method, params).await {
            Ok(CoreResponse::Json(result)) => {
                self.write(&RpcResult {
                    jsonrpc: "2.0",
                    id,
                    result,
                })
                .await
            }
            Ok(CoreResponse::Secret(secret)) => {
                self.write(&RpcResult {
                    jsonrpc: "2.0",
                    id,
                    result: secret.as_str(),
                })
                .await
            }
            Err(()) => {
                self.write(&RpcError {
                    jsonrpc: "2.0",
                    id,
                    error: RpcErrorBody {
                        code: -32601,
                        message: "core_method_unavailable",
                    },
                })
                .await
            }
        }
    }

    async fn write(&mut self, message: &impl Serialize) -> Result<(), String> {
        let mut bytes = Zeroizing::new(
            serde_json::to_vec(message)
                .map_err(|_| "Message vers l'hôte d'extensions invalide.".to_string())?,
        );
        if bytes.len() >= MAX_MESSAGE_BYTES {
            return Err("Message vers l'hôte d'extensions trop volumineux.".to_string());
        }
        bytes.push(b'\n');
        self.stdin
            .write_all(&bytes)
            .await
            .map_err(|_| "Hôte d'extensions indisponible.".to_string())?;
        self.stdin
            .flush()
            .await
            .map_err(|_| "Hôte d'extensions indisponible.".to_string())
    }
}

async fn read_bounded_line(reader: &mut BufReader<ChildStdout>) -> Result<Vec<u8>, String> {
    let mut line = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .await
            .map_err(|_| "Hôte d'extensions indisponible.".to_string())?;
        if available.is_empty() {
            return Err("L'hôte d'extensions s'est arrêté.".to_string());
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if line.len().saturating_add(take) > MAX_MESSAGE_BYTES {
            return Err("Réponse de l'hôte d'extensions trop volumineuse.".to_string());
        }
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if line.last() == Some(&b'\n') {
            line.pop();
            return Ok(line);
        }
    }
}
