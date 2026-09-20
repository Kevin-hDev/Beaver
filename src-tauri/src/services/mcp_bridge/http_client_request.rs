use std::borrow::Cow;
use std::collections::HashMap;

use reqwest::{
    header::{HeaderName, HeaderValue},
    Method, RequestBuilder,
};
use rmcp::transport::streamable_http_client::StreamableHttpError;

use super::http_client::BeaverHttpClient;
use crate::services::secure_http::SecureHttpError;

pub(super) type WireError = StreamableHttpError<SecureHttpError>;

pub(super) fn refused() -> WireError {
    WireError::UnexpectedServerResponse(Cow::Borrowed("requête MCP refusée"))
}

impl BeaverHttpClient {
    pub(super) fn request(
        &self,
        method: Method,
        uri: &str,
        auth_header: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<RequestBuilder, WireError> {
        if !self.accepts(uri) || auth_header.is_some() || headers.len() > 128 {
            return Err(refused());
        }
        let mut request = match method {
            Method::GET => self.client.get(uri),
            Method::POST => self.client.post(uri),
            Method::DELETE => self.client.delete(uri),
            _ => return Err(refused()),
        };
        for (name, value) in headers {
            if name.as_str().len() > 128
                || value.as_bytes().len() > 4096
                || [
                    "authorization",
                    "accept",
                    "content-type",
                    "mcp-session-id",
                    "last-event-id",
                ]
                .iter()
                .any(|reserved| name.as_str().eq_ignore_ascii_case(reserved))
            {
                return Err(refused());
            }
            request = request.header(name, value);
        }
        // reqwest marks bearer_auth sensitive; no SDK-owned token copy is created.
        Ok(request.bearer_auth(self.token.as_str()))
    }

    pub(super) async fn send(
        &self,
        request: RequestBuilder,
    ) -> Result<reqwest::Response, WireError> {
        tokio::select! {
            _ = self.cancellation.cancelled() => Err(WireError::Client(SecureHttpError::Request)),
            result = self.client.send(request) => result.map_err(WireError::Client),
        }
    }
}
