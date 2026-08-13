use serde_json::Value;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use super::identity;
use super::process_manager::{self, ProcessHandle};
use super::transport::next_id;
use crate::services::work_registry::ServiceWorkCancellation;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(360);
const MAX_LINE_BYTES: usize = 1_048_576;

#[derive(Clone)]
pub(super) struct StdioSession {
    connector_id: String,
    handle: ProcessHandle,
}

impl StdioSession {
    pub(super) fn new(connector_id: String, handle: ProcessHandle) -> Self {
        Self {
            connector_id,
            handle,
        }
    }

    pub(super) async fn initialize(&self, cancel: &ServiceWorkCancellation) -> Result<(), String> {
        let id = next_id();
        let request = serde_json::json!({
            "jsonrpc": "2.0", "method": "initialize", "id": id,
            "params": {
                "protocolVersion": "2025-03-26", "capabilities": {},
                "clientInfo": identity::client_info()
            }
        });
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let session = self.clone();
        let owned_cancel = cancel.clone();
        let mut reader_tasks = tokio::task::JoinSet::new();
        reader_tasks.spawn(async move {
            let result = session.request(&request, id, &owned_cancel).await;
            let _ = ready_tx.send(result);
        });

        // The protocol response is the sole readiness authority; owning this task
        // guarantees cancellation cannot leave a detached reader behind.
        let readiness = tokio::select! {
            result = ready_rx => result.map_err(|_| "connecteur MCP indisponible".to_string())?,
            _ = cancel.cancelled() => Err("connecteur MCP indisponible".to_string()),
        };
        reader_tasks.abort_all();
        while reader_tasks.join_next().await.is_some() {}
        readiness?;

        self.write_line(
            &serde_json::json!({
                "jsonrpc": "2.0", "method": "notifications/initialized"
            }),
            cancel,
        )
        .await
    }

    pub(super) async fn request(
        &self,
        request: &Value,
        expected_id: u64,
        cancel: &ServiceWorkCancellation,
    ) -> Result<Value, String> {
        let guard = tokio::select! {
            guard = self.handle.request_lock.lock() => guard,
            _ = cancel.cancelled() => return Err("connecteur MCP indisponible".to_string()),
        };
        self.write_line(request, cancel).await?;
        let response = self.read_response(expected_id, cancel).await;
        drop(guard);
        response
    }

    async fn write_line(
        &self,
        message: &Value,
        cancel: &ServiceWorkCancellation,
    ) -> Result<(), String> {
        let mut line = serde_json::to_string(message).map_err(|_| "requête MCP invalide")?;
        line.push('\n');
        let mut stdin = tokio::select! {
            stdin = self.handle.stdin.lock() => stdin,
            _ = cancel.cancelled() => return Err("connecteur MCP indisponible".to_string()),
        };
        let writer = stdin
            .as_mut()
            .ok_or_else(|| "connecteur MCP indisponible".to_string())?;
        tokio::select! {
            result = writer.write_all(line.as_bytes()) => {
                result.map_err(|_| "connecteur MCP indisponible".to_string())?;
            }
            _ = cancel.cancelled() => return Err("connecteur MCP indisponible".to_string()),
        }
        tokio::select! {
            result = writer.flush() => {
                result.map_err(|_| "connecteur MCP indisponible".to_string())?;
            }
            _ = cancel.cancelled() => return Err("connecteur MCP indisponible".to_string()),
        }
        Ok(())
    }

    async fn read_response(
        &self,
        expected_id: u64,
        cancel: &ServiceWorkCancellation,
    ) -> Result<Value, String> {
        let mut reader = tokio::select! {
            reader = self.handle.reader.lock() => reader,
            _ = cancel.cancelled() => return Err("connecteur MCP indisponible".to_string()),
        };
        let result = {
            let response = tokio::time::timeout(REQUEST_TIMEOUT, async {
                loop {
                    let line = super::stdio_line::read_bounded_line(&mut *reader, MAX_LINE_BYTES)
                        .await?
                        .ok_or_else(|| "connecteur MCP indisponible".to_string())?;
                    let text = std::str::from_utf8(&line)
                        .map_err(|_| "réponse MCP invalide".to_string())?;
                    let trimmed = text.trim();
                    if !trimmed.starts_with('{') || !trimmed.contains("\"jsonrpc\"") {
                        continue;
                    }
                    let parsed: Value = serde_json::from_str(trimmed)
                        .map_err(|_| "réponse MCP invalide".to_string())?;
                    if parsed.get("id").and_then(Value::as_u64) == Some(expected_id) {
                        return Ok::<Value, String>(parsed);
                    }
                }
            });
            tokio::pin!(response);
            tokio::select! {
                result = &mut response => Some(result),
                _ = cancel.cancelled() => None,
            }
        };
        drop(reader);
        match result {
            Some(Ok(Ok(value))) => Ok(value),
            Some(Ok(Err(_))) | Some(Err(_)) | None => {
                process_manager::shutdown_one(&self.connector_id).await;
                Err("réponse MCP invalide".to_string())
            }
        }
    }
}
