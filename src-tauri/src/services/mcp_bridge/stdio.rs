use serde_json::Value;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use zeroize::Zeroizing;

use super::identity;
use super::process_manager::{self, ProcessHandle};
use super::transport::next_id;
use crate::services::work_registry::ServiceWorkCancellation;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(360);
const MAX_LINE_BYTES: usize = 1_048_576;
const WARMUP_MS: u64 = 500;

pub struct StdioTransport {
    pub connector_id: String,
    pub install_command: String,
    pub env_key_names: Vec<String>,
    pub transient_env: Option<Vec<(String, Zeroizing<String>)>>,
    #[cfg(test)]
    pub(super) test_fixture: bool,
}

impl StdioTransport {
    async fn ensure_running(
        &self,
        cancel: &ServiceWorkCancellation,
    ) -> Result<ProcessHandle, String> {
        let env_tokens = self.resolve_env_tokens();
        #[cfg(test)]
        let handle = if self.test_fixture {
            process_manager::ensure_test_fixture(&self.connector_id).await?
        } else {
            self.ensure_configured_process(&env_tokens).await?
        };
        #[cfg(not(test))]
        let handle = self.ensure_configured_process(&env_tokens).await?;

        let initialized = handle
            .initialized
            .get_or_try_init(|| async {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(WARMUP_MS)) => {}
                    _ = cancel.cancelled() => {
                        return Err("connecteur MCP indisponible".to_string());
                    }
                }
                self.handshake(&handle, cancel).await
            })
            .await;
        if let Err(error) = initialized {
            process_manager::shutdown_one(&self.connector_id).await;
            return Err(error);
        }
        Ok(handle)
    }

    async fn ensure_configured_process(
        &self,
        env_tokens: &[(String, Zeroizing<String>)],
    ) -> Result<ProcessHandle, String> {
        process_manager::ensure_process(
            &self.connector_id,
            &self.install_command,
            env_tokens,
            self.transient_env.is_some(),
        )
        .await
    }

    async fn handshake(
        &self,
        handle: &ProcessHandle,
        cancel: &ServiceWorkCancellation,
    ) -> Result<(), String> {
        let id = next_id();
        let init = serde_json::json!({
            "jsonrpc": "2.0", "method": "initialize", "id": id,
            "params": {
                "protocolVersion": "2025-03-26", "capabilities": {},
                "clientInfo": identity::client_info()
            }
        });
        let _ = self.send_with_id(handle, &init, id, cancel).await?;
        self.write_line(
            handle,
            &serde_json::json!({
                "jsonrpc": "2.0", "method": "notifications/initialized"
            }),
            cancel,
        )
        .await
    }

    async fn send_with_id(
        &self,
        handle: &ProcessHandle,
        request: &Value,
        expected_id: u64,
        cancel: &ServiceWorkCancellation,
    ) -> Result<Value, String> {
        let guard = tokio::select! {
            guard = handle.request_lock.lock() => guard,
            _ = cancel.cancelled() => return Err("connecteur MCP indisponible".to_string()),
        };
        self.write_line(handle, request, cancel).await?;
        let response = self.read_response(handle, Some(expected_id), cancel).await;
        drop(guard);
        response
    }

    async fn write_line(
        &self,
        handle: &ProcessHandle,
        message: &Value,
        cancel: &ServiceWorkCancellation,
    ) -> Result<(), String> {
        let mut line = serde_json::to_string(message).map_err(|_| "requête MCP invalide")?;
        line.push('\n');
        let mut stdin = tokio::select! {
            stdin = handle.stdin.lock() => stdin,
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
        handle: &ProcessHandle,
        expected_id: Option<u64>,
        cancel: &ServiceWorkCancellation,
    ) -> Result<Value, String> {
        let mut reader = tokio::select! {
            reader = handle.reader.lock() => reader,
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
                    if expected_id
                        .is_none_or(|id| parsed.get("id").and_then(Value::as_u64) == Some(id))
                    {
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

include!("stdio_transport.rs");
