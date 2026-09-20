pub struct HttpTransport {
    pub connector_id: String,
    pub endpoint: String,
    pub transient_token: Option<zeroize::Zeroizing<String>>,
    pub(super) generation: Option<u64>,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[async_trait::async_trait]
impl super::transport::McpTransport for HttpTransport {
    async fn list_tools(&self) -> Result<super::transport::McpToolCatalog, String> {
        let token = self.resolve_token().await?;
        super::http_lifecycle::list_tools(&self.connector_id, &self.endpoint, Some(&token)).await
    }

    async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<super::transport::McpToolResult, super::transport::McpCallError> {
        let token = self.resolve_token_for_call().await?;
        match self.generation {
            Some(generation) => {
                super::http_lifecycle::call_tool_checked(
                    &self.connector_id,
                    &self.endpoint,
                    Some(&token),
                    name,
                    args,
                    Some(generation),
                )
                .await
            }
            None => {
                super::http_lifecycle::call_tool(
                    &self.connector_id,
                    &self.endpoint,
                    Some(&token),
                    name,
                    args,
                )
                .await
            }
        }
    }
}
