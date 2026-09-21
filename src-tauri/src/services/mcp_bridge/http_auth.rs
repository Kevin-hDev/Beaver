use zeroize::Zeroizing;

use super::http::HttpTransport;

impl HttpTransport {
    pub fn new(connector_id: String, endpoint: String) -> Self {
        Self {
            connector_id,
            endpoint,
            transient_token: None,
            generation: None,
        }
    }

    pub fn new_with_token(
        connector_id: String,
        endpoint: String,
        token: Zeroizing<String>,
    ) -> Self {
        Self {
            connector_id,
            endpoint,
            transient_token: Some(token),
            generation: None,
        }
    }

    pub(super) async fn resolve_token(&self) -> Result<Zeroizing<String>, String> {
        match &self.transient_token {
            Some(token) => Ok(token.clone()),
            None => crate::services::mcp_oauth::storage::get_valid_token(&self.connector_id).await,
        }
    }

    pub(super) async fn resolve_token_for_call(
        &self,
    ) -> Result<Zeroizing<String>, super::transport::McpCallError> {
        self.resolve_token().await.map_err(|error| {
            if error == crate::services::mcp_oauth::types::REAUTHENTICATION_REQUIRED {
                super::transport::McpCallError::ReauthenticationRequired
            } else {
                super::transport::McpCallError::Unavailable
            }
        })
    }
}
