use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use zeroize::Zeroizing;

use super::identity;
use super::process_manager::{self, ProcessHandle};
use super::transport::{next_id, validate_tools, McpToolDef, McpTransport};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(360);
const MAX_LINE_BYTES: usize = 1_048_576;
const WARMUP_MS: u64 = 500;

pub struct StdioTransport {
    pub connector_id: String,
    pub install_command: String,
    pub env_key_names: Vec<String>,
    pub transient_env: Option<Vec<(String, Zeroizing<String>)>>,
}

impl StdioTransport {
    async fn ensure_running(&self) -> Result<ProcessHandle, String> {
        if self.transient_env.is_none() {
            if let Some(handle) = process_manager::get_alive_handle(&self.connector_id) {
                return Ok(handle);
            }
        }

        process_manager::shutdown_one(&self.connector_id);
        let env_tokens = self.resolve_env_tokens();
        let handle =
            process_manager::spawn(&self.connector_id, &self.install_command, &env_tokens)?;
        tokio::time::sleep(Duration::from_millis(WARMUP_MS)).await;
        self.handshake(&handle).await?;
        Ok(handle)
    }

    async fn handshake(&self, handle: &ProcessHandle) -> Result<(), String> {
        let id = next_id();
        let init = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": id,
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": identity::client_info()
            }
        });

        let _ = self.send_with_id(handle, &init, id).await?;

        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        self.write_line(handle, &notif).await?;
        Ok(())
    }

    async fn send_with_id(
        &self,
        handle: &ProcessHandle,
        request: &Value,
        expected_id: u64,
    ) -> Result<Value, String> {
        let _guard = handle.request_lock.lock().await;
        self.write_line(handle, request).await?;
        self.read_response(handle, Some(expected_id)).await
    }

    async fn write_line(&self, handle: &ProcessHandle, msg: &Value) -> Result<(), String> {
        let mut line = serde_json::to_string(msg).map_err(|_| "sérialisation échouée")?;
        line.push('\n');

        let mut stdin = handle.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|_| "impossible d'écrire sur stdin du process MCP".to_string())?;
        stdin.flush().await.map_err(|_| "flush stdin échoué")?;
        Ok(())
    }

    async fn read_response(
        &self,
        handle: &ProcessHandle,
        expected_id: Option<u64>,
    ) -> Result<Value, String> {
        let mut reader = handle.reader.lock().await;

        let result = tokio::time::timeout(REQUEST_TIMEOUT, async {
            loop {
                let line = super::stdio_line::read_bounded_line(&mut *reader, MAX_LINE_BYTES)
                    .await?
                    .ok_or_else(|| "le process MCP s'est arrêté".to_string())?;
                let text = std::str::from_utf8(&line)
                    .map_err(|_| "réponse JSON-RPC invalide".to_string())?;
                let trimmed = text.trim();
                if !trimmed.starts_with('{') || !trimmed.contains("\"jsonrpc\"") {
                    continue;
                }
                let parsed: Value = serde_json::from_str(trimmed)
                    .map_err(|_| "réponse JSON-RPC invalide".to_string())?;
                if let Some(eid) = expected_id {
                    if parsed.get("id").and_then(Value::as_u64) != Some(eid) {
                        continue;
                    }
                }
                return Ok::<Value, String>(parsed);
            }
        })
        .await;
        drop(reader);

        match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) | Err(_) => {
                process_manager::shutdown_one(&self.connector_id);
                Err("réponse MCP invalide".to_string())
            }
        }
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn list_tools(&self) -> Result<Vec<McpToolDef>, String> {
        let handle = self.ensure_running().await?;
        let id = next_id();

        let body = serde_json::json!({
            "jsonrpc": "2.0", "method": "tools/list", "id": id
        });

        let resp = self.send_with_id(&handle, &body, id).await?;

        let tools_val = resp
            .get("result")
            .and_then(|r| r.get("tools").cloned())
            .ok_or("réponse tools/list invalide")?;

        let tools: Vec<McpToolDef> =
            serde_json::from_value(tools_val).map_err(|_| "format tools invalide")?;

        validate_tools(tools)
    }

    async fn call_tool(
        &self,
        name: &str,
        args: Value,
    ) -> Result<super::transport::McpToolResult, String> {
        let handle = self.ensure_running().await?;
        let id = next_id();

        let body = serde_json::json!({
            "jsonrpc": "2.0", "method": "tools/call", "id": id,
            "params": { "name": name, "arguments": args }
        });

        let resp = self.send_with_id(&handle, &body, id).await?;
        super::transport::extract_tool_result(&resp)
    }
}
