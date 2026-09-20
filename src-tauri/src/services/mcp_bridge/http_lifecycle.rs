use std::sync::Arc;
use std::time::{Duration, Instant};

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, ClientConfig, Implementation, ProtocolVersion,
};
use rmcp::service::{ClientCacheConfig, ClientLifecycleMode, ClientServiceExt, RunningService};
use rmcp::transport::common::client_side_sse::NeverRetry;
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use rmcp::RoleClient;
use serde_json::Value;

use super::http_catalog::list;
use super::http_client::BeaverHttpClient;
use super::transport::{McpCallError, McpToolCatalog, McpToolResult};

const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
// Keep shutdown inside the operation budget, even when the request uses its full allowance.
const SHUTDOWN_RESERVE: Duration = Duration::from_secs(5);

pub(super) async fn start_with_client(
    client: BeaverHttpClient,
    endpoint: &str,
) -> Result<RunningService<RoleClient, ClientConfig>, Box<rmcp::service::ClientInitializeError>> {
    let cancel_client_on_drop = client.cancellation.clone().drop_guard();
    let mut transport_config = StreamableHttpClientTransportConfig::with_uri(endpoint);
    transport_config.auth_header = None;
    transport_config.retry_config = Arc::new(NeverRetry::default());
    transport_config.reinit_on_expired_session = false;
    transport_config.max_sse_event_size = crate::services::secure_http::MCP_BODY_LIMIT;
    let transport = StreamableHttpClientTransport::with_client(client, transport_config);

    let mut identity = ClientConfig::default();
    identity.protocol_version = ProtocolVersion::V_2025_03_26;
    identity.client_info = Implementation::new(
        crate::services::brand::MCP_CLIENT_NAME,
        env!("CARGO_PKG_VERSION"),
    );
    let lifecycle = ClientLifecycleMode::Auto {
        preferred_versions: vec![ProtocolVersion::V_2026_07_28, ProtocolVersion::V_2025_11_25],
        legacy_version: Some(ProtocolVersion::V_2025_03_26),
    };
    let service = identity
        .serve_with_lifecycle(transport, lifecycle)
        .await
        .map_err(Box::new)?;
    cancel_client_on_drop.disarm();
    service
        .set_response_cache_config(ClientCacheConfig::disabled())
        .await;
    Ok(service)
}

pub(super) async fn list_tools(
    connector_id: &str,
    endpoint: &str,
    token: Option<&str>,
) -> Result<McpToolCatalog, String> {
    let token = zeroize::Zeroizing::new(token.ok_or("autorisation MCP absente")?.to_owned());
    let client = BeaverHttpClient::new(connector_id, endpoint, &token)?;
    list_tools_with_client(client, endpoint.to_owned()).await
}

pub(super) async fn list_tools_with_client(
    client: BeaverHttpClient,
    endpoint: String,
) -> Result<McpToolCatalog, String> {
    let (mut sender, receiver) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let deadline = Instant::now() + OPERATION_TIMEOUT - SHUTDOWN_RESERVE;
        let result = async {
            let _admission = super::process_manager::try_admit_operation()
                .map_err(|_| "connecteur MCP indisponible".to_string())?;
            let mut service = tokio::select! {
                _ = sender.closed() => return Err("opération MCP annulée".to_string()),
                result = tokio::time::timeout_at(
                    tokio::time::Instant::from_std(deadline),
                    start_with_client(client, &endpoint),
                ) => result
                    .map_err(|_| "délai MCP dépassé".to_string())?
                    .map_err(|_| "connexion MCP indisponible".to_string())?,
            };
            let result = tokio::select! {
                _ = sender.closed() => Err("opération MCP annulée".to_string()),
                result = tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), list(&service)) =>
                    result.unwrap_or_else(|_| Err("délai MCP dépassé".to_string())),
            };
            if service.close().await.is_ok() { result } else { Err("arrêt MCP incomplet".to_string()) }
        }.await;
        let _ = sender.send(result);
    });
    tokio::time::timeout(OPERATION_TIMEOUT, receiver)
        .await
        .map_err(|_| "délai MCP dépassé".to_string())?
        .map_err(|_| "opération MCP interrompue".to_string())?
}

pub(super) async fn call_tool(
    connector_id: &str,
    endpoint: &str,
    token: Option<&str>,
    name: &str,
    args: Value,
) -> Result<McpToolResult, McpCallError> {
    call_tool_checked(connector_id, endpoint, token, name, args, None).await
}

pub(super) async fn call_tool_checked(
    connector_id: &str,
    endpoint: &str,
    token: Option<&str>,
    name: &str,
    args: Value,
    generation: Option<u64>,
) -> Result<McpToolResult, McpCallError> {
    let token = zeroize::Zeroizing::new(token.ok_or(McpCallError::Unavailable)?.to_owned());
    let client = BeaverHttpClient::new(connector_id, endpoint, &token)
        .map_err(|_| McpCallError::Unavailable)?;
    let checked_id = connector_id.to_owned();
    call_tool_with_client(
        client,
        endpoint.to_owned(),
        name,
        args,
        move || match generation {
            Some(generation) => super::registry::authorize_business_send(&checked_id, generation),
            None => Ok(()),
        },
    )
    .await
}

pub(super) async fn call_tool_with_client<F>(
    client: BeaverHttpClient,
    endpoint: String,
    name: &str,
    args: Value,
    authorize: F,
) -> Result<McpToolResult, McpCallError>
where
    F: Fn() -> Result<(), McpCallError> + Send + Sync + 'static,
{
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(McpCallError::InvalidResponse);
    }
    super::schema_limits::validate(&args).map_err(|_| McpCallError::InvalidResponse)?;
    let arguments = args
        .as_object()
        .cloned()
        .ok_or(McpCallError::InvalidResponse)?;
    let name = name.to_owned();
    let (mut sender, receiver) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let deadline = Instant::now() + OPERATION_TIMEOUT - SHUTDOWN_RESERVE;
        let result = async {
            let _admission = super::process_manager::try_admit_operation()
                .map_err(|_| McpCallError::Unavailable)?;
            let mut service = tokio::select! {
                _ = sender.closed() => return Err(McpCallError::Transport),
                result = tokio::time::timeout_at(
                    tokio::time::Instant::from_std(deadline),
                    start_with_client(client, &endpoint),
                ) => result.map_err(|_| McpCallError::Transport)?
                    .map_err(|_| McpCallError::Unavailable)?,
            };
            let mut params = CallToolRequestParams::new(name);
            params.arguments = Some(arguments);
            let result = if authorize().is_err() {
                Err(McpCallError::Unavailable)
            } else {
                tokio::select! {
                    _ = sender.closed() => Err(McpCallError::Transport),
                    result = tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), service.call_tool_once(params)) => {
                        match result {
                            Ok(Ok(CallToolResponse::Complete(value))) => {
                                serde_json::to_value(value)
                                    .map_err(|_| McpCallError::InvalidResponse)
                                    .and_then(|value| super::result::complete(&value))
                            }
                            Ok(Ok(_)) => Err(McpCallError::InvalidResponse),
                            _ => Err(McpCallError::Transport),
                        }
                    }
                }
            };
            if service.close().await.is_ok() { result } else { Err(McpCallError::Transport) }
        }.await;
        let _ = sender.send(result);
    });
    tokio::time::timeout(OPERATION_TIMEOUT, receiver)
        .await
        .map_err(|_| McpCallError::Transport)?
        .map_err(|_| McpCallError::Transport)?
}
