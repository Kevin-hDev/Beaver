use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use super::identity;
use super::response;
use super::transport::{
    next_id, validate_tools, McpCallError, McpToolDef, McpToolResult, McpTransport,
};
use crate::services::secure_http::AuthenticatedClient;

const TIMEOUT: Duration = Duration::from_secs(30);
const ACCEPT: &str = "application/json, text/event-stream";

pub struct HttpTransport {
    pub connector_id: String,
    pub endpoint: String,
    pub transient_token: Option<zeroize::Zeroizing<String>>,
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn list_tools(&self) -> Result<Vec<McpToolDef>, String> {
        let token = self.resolve_token().await?;
        let session_id = initialize(&self.endpoint, token.as_str()).await?;

        let body = serde_json::json!({
            "jsonrpc": "2.0", "method": "tools/list", "id": next_id()
        });

        let resp = mcp_post(&self.endpoint, token.as_str(), session_id.as_deref(), &body).await?;

        let tools_val = resp
            .result
            .and_then(|r| r.get("tools").cloned())
            .ok_or("réponse tools/list invalide")?;

        let tools: Vec<McpToolDef> =
            serde_json::from_value(tools_val).map_err(|_| "format tools invalide")?;

        validate_tools(tools)
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<McpToolResult, McpCallError> {
        let token = self
            .resolve_token()
            .await
            .map_err(|_| McpCallError::Unavailable)?;
        let session_id = initialize(&self.endpoint, token.as_str())
            .await
            .map_err(|_| McpCallError::Unavailable)?;

        let body = serde_json::json!({
            "jsonrpc": "2.0", "method": "tools/call", "id": next_id(),
            "params": { "name": name, "arguments": args }
        });

        let resp = mcp_post(&self.endpoint, token.as_str(), session_id.as_deref(), &body)
            .await
            .map_err(|_| McpCallError::Transport)?;

        if resp.error.is_some() {
            return Err(McpCallError::Server);
        }

        let result = resp.result.ok_or(McpCallError::InvalidResponse)?;
        super::transport::extract_tool_result(&serde_json::json!({ "result": result }))
    }
}

async fn initialize(endpoint: &str, token: &str) -> Result<Option<String>, String> {
    let init_body = serde_json::json!({
        "jsonrpc": "2.0", "method": "initialize", "id": next_id(),
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": identity::client_info()
        }
    });

    let client = build_client()?;

    let request = client
        .post(endpoint)
        .bearer_auth(token)
        .header("Accept", ACCEPT)
        .header("Content-Type", "application/json")
        .json(&init_body);
    let resp = client
        .send_success(request)
        .await
        .map_err(|_| "impossible de contacter le serveur MCP")?;

    let session_id = resp
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| s.len() <= 256 && s.bytes().all(|b| (0x20..0x7f).contains(&b)))
        .map(|s| s.to_string());

    let _ = response::parse(resp).await?;

    let notif = serde_json::json!({
        "jsonrpc": "2.0", "method": "notifications/initialized"
    });

    let mut req = client
        .post(endpoint)
        .bearer_auth(token)
        .header("Accept", ACCEPT)
        .header("Content-Type", "application/json");

    if let Some(ref sid) = session_id {
        req = req.header("Mcp-Session-Id", sid);
    }

    client
        .send_success(req.json(&notif))
        .await
        .map_err(|_| "notification initialized échouée".to_string())?;
    Ok(session_id)
}

async fn mcp_post(
    endpoint: &str,
    token: &str,
    session_id: Option<&str>,
    body: &Value,
) -> Result<response::JsonRpcResponse, String> {
    let client = build_client()?;

    let mut req = client
        .post(endpoint)
        .bearer_auth(token)
        .header("Accept", ACCEPT)
        .header("Content-Type", "application/json");

    if let Some(sid) = session_id {
        req = req.header("Mcp-Session-Id", sid);
    }

    let resp = client
        .send_success(req.json(body))
        .await
        .map_err(|_| "impossible de contacter le serveur MCP")?;

    response::parse(resp).await
}

fn build_client() -> Result<AuthenticatedClient, String> {
    AuthenticatedClient::new(TIMEOUT).map_err(|_| "erreur interne".to_string())
}
