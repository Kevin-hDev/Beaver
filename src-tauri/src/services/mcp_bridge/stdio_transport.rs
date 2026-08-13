use async_trait::async_trait;
use serde_json::Value;

use super::process_manager;
use super::stdio::StdioTransport;
use super::stdio_session::StdioSession;
use super::transport::next_id;
use crate::services::work_registry::ServiceWorkCancellation;

impl StdioTransport {
    async fn ensure_running(
        &self,
        cancel: &ServiceWorkCancellation,
    ) -> Result<StdioSession, String> {
        let env_tokens = self.resolve_env_tokens();
        #[cfg(test)]
        let handle = if self.connector_id.starts_with("__beaver_mcp_") {
            process_manager::ensure_test_fixture(&self.connector_id, self.test_init_delay_ms)
                .await?
        } else {
            self.ensure_configured_process(&env_tokens).await?
        };
        #[cfg(not(test))]
        let handle = self.ensure_configured_process(&env_tokens).await?;

        let session = StdioSession::new(self.connector_id.clone(), handle.clone());
        let initialized = handle
            .initialized
            .get_or_try_init(|| session.initialize(cancel))
            .await;
        if let Err(error) = initialized {
            process_manager::shutdown_one(&self.connector_id).await;
            return Err(error);
        }
        Ok(session)
    }

    async fn ensure_configured_process(
        &self,
        env_tokens: &[(String, zeroize::Zeroizing<String>)],
    ) -> Result<super::process_manager::ProcessHandle, String> {
        process_manager::ensure_process(
            &self.connector_id,
            &self.install_command,
            env_tokens,
            self.transient_env.is_some(),
        )
        .await
    }

    async fn list_tools_inner(
        &self,
        cancel: &ServiceWorkCancellation,
    ) -> Result<Vec<super::transport::McpToolDef>, String> {
        let session = self.ensure_running(cancel).await?;
        let id = next_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0", "method": "tools/list", "id": id
        });
        let response = session.request(&body, id, cancel).await?;
        let tools = response
            .get("result")
            .and_then(|result| result.get("tools").cloned())
            .ok_or("réponse tools/list invalide")?;
        super::transport::validate_tools(
            serde_json::from_value(tools).map_err(|_| "format tools invalide")?,
        )
    }

    async fn call_tool_inner(
        &self,
        name: &str,
        args: Value,
        cancel: &ServiceWorkCancellation,
    ) -> Result<super::transport::McpToolResult, super::transport::McpCallError> {
        let session = self
            .ensure_running(cancel)
            .await
            .map_err(|_| super::transport::McpCallError::Unavailable)?;
        let id = next_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0", "method": "tools/call", "id": id,
            "params": { "name": name, "arguments": args }
        });
        let response = session
            .request(&body, id, cancel)
            .await
            .map_err(|_| super::transport::McpCallError::Transport)?;
        super::transport::extract_tool_result(&response)
    }
}

#[async_trait]
impl super::transport::McpTransport for StdioTransport {
    async fn list_tools(&self) -> Result<Vec<super::transport::McpToolDef>, String> {
        let admission = process_manager::try_admit_operation()
            .map_err(|_| "connecteur MCP indisponible".to_string())?;
        let cancel = admission.cancellation();
        admission.run(self.list_tools_inner(&cancel)).await
    }

    async fn call_tool(
        &self,
        name: &str,
        args: Value,
    ) -> Result<super::transport::McpToolResult, super::transport::McpCallError> {
        let admission = process_manager::try_admit_operation()
            .map_err(|_| super::transport::McpCallError::Unavailable)?;
        let cancel = admission.cancellation();
        admission
            .run(self.call_tool_inner(name, args, &cancel))
            .await
    }
}
