use async_trait::async_trait;

impl StdioTransport {
    async fn list_tools_inner(
        &self,
        cancel: &ServiceWorkCancellation,
    ) -> Result<Vec<super::transport::McpToolDef>, String> {
        let handle = self.ensure_running(cancel).await?;
        let id = next_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0", "method": "tools/list", "id": id
        });
        let response = self.send_with_id(&handle, &body, id, cancel).await?;
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
        let handle = self
            .ensure_running(cancel)
            .await
            .map_err(|_| super::transport::McpCallError::Unavailable)?;
        let id = next_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0", "method": "tools/call", "id": id,
            "params": { "name": name, "arguments": args }
        });
        let response = self
            .send_with_id(&handle, &body, id, cancel)
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
