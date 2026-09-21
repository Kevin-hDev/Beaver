use std::collections::HashMap;
use std::sync::{atomic::AtomicUsize, Arc};

use futures_util::stream::BoxStream;
use reqwest::{
    header::{HeaderName, HeaderValue},
    Method, Url,
};
use rmcp::model::{ClientJsonRpcMessage, ClientRequest};
use rmcp::transport::streamable_http_client::{
    SseError, StreamableHttpClient, StreamableHttpPostResponse,
};
use sse_stream::Sse;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

use super::http_client_request::{refused, WireError};
use crate::services::secure_http::{AuthenticatedClient, SecureHttpError, MCP_BODY_LIMIT};

#[derive(Clone)]
pub(super) struct BeaverHttpClient {
    pub(super) client: AuthenticatedClient,
    pub(super) endpoint: Url,
    pub(super) token: Arc<Zeroizing<String>>,
    pub(super) bytes_left: Arc<AtomicUsize>,
    pub(super) cancellation: CancellationToken,
}

impl BeaverHttpClient {
    pub(super) fn new(connector_id: &str, endpoint: &str, token: &str) -> Result<Self, String> {
        if !super::trusted::is_trusted_endpoint_for_connector(connector_id, endpoint) {
            return Err("adresse MCP refusée".to_string());
        }
        let endpoint = Url::parse(endpoint).map_err(|_| "adresse MCP refusée".to_string())?;
        if endpoint.username() != ""
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err("adresse MCP refusée".to_string());
        }
        let client = AuthenticatedClient::new_streaming(
            std::time::Duration::from_secs(10),
            std::time::Duration::from_secs(30),
        )
        .map_err(|_| "client MCP indisponible".to_string())?;
        Ok(Self {
            client,
            endpoint,
            token: Arc::new(Zeroizing::new(token.to_owned())),
            bytes_left: Arc::new(AtomicUsize::new(MCP_BODY_LIMIT)),
            cancellation: CancellationToken::new(),
        })
    }

    pub(super) fn accepts(&self, uri: &str) -> bool {
        Url::parse(uri).is_ok_and(|requested| {
            requested.username().is_empty()
                && requested.password().is_none()
                && requested.query().is_none()
                && requested.fragment().is_none()
                && requested == self.endpoint
        })
    }
}

impl StreamableHttpClient for BeaverHttpClient {
    type Error = SecureHttpError;

    async fn post_message(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session_id: Option<Arc<str>>,
        auth_header: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<StreamableHttpPostResponse, WireError> {
        self.post_message_with_max_sse_event_size(
            uri,
            message,
            session_id,
            auth_header,
            headers,
            MCP_BODY_LIMIT,
        )
        .await
    }

    async fn post_message_with_max_sse_event_size(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session_id: Option<Arc<str>>,
        auth_header: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
        event_limit: usize,
    ) -> Result<StreamableHttpPostResponse, WireError> {
        let is_request = matches!(message, ClientJsonRpcMessage::Request(_));
        let is_discovery = matches!(
            &message,
            ClientJsonRpcMessage::Request(request)
                if matches!(request.request, ClientRequest::DiscoverRequest(_))
        );
        let mut request = self
            .request(Method::POST, &uri, auth_header, headers)?
            .header("Accept", "application/json, text/event-stream");
        if let Some(session) = &session_id {
            if session.len() > 256 {
                return Err(refused());
            }
            request = request.header("Mcp-Session-Id", session.as_ref());
        }
        // The SDK falls back after 10 s; fail this request first instead of
        // treating a slow modern server as proof that it is legacy.
        let request = if is_discovery {
            request.timeout(std::time::Duration::from_secs(9))
        } else {
            request
        };
        let response = self.send(request.json(&message)).await?;
        super::http_client_response::post(
            response,
            session_id.is_some(),
            is_request,
            is_discovery,
            event_limit,
            self.bytes_left.clone(),
        )
        .await
    }

    async fn delete_session(
        &self,
        uri: Arc<str>,
        session_id: Arc<str>,
        auth_header: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<(), WireError> {
        if session_id.len() > 256 {
            return Err(refused());
        }
        let request = self
            .request(Method::DELETE, &uri, auth_header, headers)?
            .header("Mcp-Session-Id", session_id.as_ref());
        super::http_client_response::delete(self.send(request).await?)
    }

    async fn get_stream(
        &self,
        uri: Arc<str>,
        session_id: Option<Arc<str>>,
        last_event_id: Option<String>,
        auth_header: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<BoxStream<'static, Result<Sse, SseError>>, WireError> {
        self.get_stream_with_max_sse_event_size(
            uri,
            session_id,
            last_event_id,
            auth_header,
            headers,
            MCP_BODY_LIMIT,
        )
        .await
    }

    async fn get_stream_with_max_sse_event_size(
        &self,
        uri: Arc<str>,
        session_id: Option<Arc<str>>,
        last_event_id: Option<String>,
        auth_header: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
        event_limit: usize,
    ) -> Result<BoxStream<'static, Result<Sse, SseError>>, WireError> {
        let mut request = self
            .request(Method::GET, &uri, auth_header, headers)?
            .header("Accept", "text/event-stream, application/json");
        if let Some(session) = session_id {
            if session.len() > 256 {
                return Err(refused());
            }
            request = request.header("Mcp-Session-Id", session.as_ref());
        }
        if let Some(event_id) = last_event_id {
            if event_id.len() > 256 {
                return Err(refused());
            }
            request = request.header("Last-Event-Id", event_id);
        }
        super::http_client_response::get(
            self.send(request).await?,
            event_limit,
            self.bytes_left.clone(),
        )
    }
}
