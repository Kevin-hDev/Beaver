use zeroize::Zeroizing;

use super::http::HttpTransport;

impl HttpTransport {
    pub fn new(connector_id: String, endpoint: String) -> Self {
        Self {
            connector_id,
            endpoint,
            transient_token: None,
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
        }
    }

    pub(super) async fn resolve_token(&self) -> Result<Zeroizing<String>, String> {
        match &self.transient_token {
            Some(token) => Ok(token.clone()),
            None => crate::services::mcp_oauth::storage::get_valid_token(&self.connector_id).await,
        }
    }
}
