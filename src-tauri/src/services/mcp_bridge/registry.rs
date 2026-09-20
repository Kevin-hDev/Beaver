use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::http::HttpTransport;
use super::registry_cache::CacheState;
use super::stdio::StdioTransport;
use super::transport::{McpCallError, McpToolDef, McpToolResult, McpTransport};
use super::{config, process_manager, token_validation, trusted};

const TEST_TIMEOUT_SECS: u64 = 20;

static CACHE_STATE: std::sync::LazyLock<Mutex<CacheState>> =
    std::sync::LazyLock::new(|| Mutex::new(CacheState::default()));

pub(crate) struct IdentityMutation(());

pub struct EnabledConnector {
    pub id: String,
    pub transport: Arc<dyn McpTransport>,
    pub generation: u64,
}

pub fn get_enabled_connectors() -> Result<Vec<EnabledConnector>, String> {
    let state = CACHE_STATE
        .lock()
        .map_err(|_| "registre MCP indisponible")?;
    if state.is_closed() {
        return Err("registre MCP indisponible".to_string());
    }
    let generation = state.generation();
    Ok(config::load()?
        .into_iter()
        .filter(|c| c.status == "connected" && c.enabled_in_chat)
        .filter_map(|connector| build_connector(connector, generation))
        .take(config::MAX_CONNECTORS)
        .collect())
}

pub fn is_trusted_endpoint_pub(connector_id: &str, url: &str) -> bool {
    trusted::is_trusted_endpoint_for_connector(connector_id, url)
}

fn build_connector(c: config::StoredConnector, generation: u64) -> Option<EnabledConnector> {
    if !config::is_valid_connector_id(&c.id) {
        return None;
    }
    if c.id == "imessage" && !cfg!(target_os = "macos") {
        return None;
    }
    if let Some(ref endpoint) = c.endpoint {
        if trusted::is_trusted_endpoint_for_connector(&c.id, endpoint) {
            let mut transport = HttpTransport::new(c.id.clone(), endpoint.clone());
            transport.generation = Some(generation);
            return Some(EnabledConnector {
                id: c.id,
                transport: Arc::new(transport),
                generation,
            });
        }
    }

    if let Some(cmd) = config::install_command_for(&c) {
        let env_key_names = config::validated_env_keys(c.env_keys.as_deref()).ok()?;
        let mut transport = StdioTransport::new(c.id.clone(), cmd, env_key_names);
        transport.generation = Some(generation);
        return Some(EnabledConnector {
            id: c.id,
            transport: Arc::new(transport),
            generation,
        });
    }

    None
}

pub async fn get_tools(connector: &EnabledConnector) -> Result<Vec<McpToolDef>, String> {
    let now = Instant::now();
    if let Some(cached) = CACHE_STATE
        .lock()
        .map_err(|_| "registre MCP indisponible")?
        .get(&connector.id, connector.generation, now)
    {
        return Ok(cached);
    }

    let mut catalog = connector.transport.list_tools().await?;
    catalog
        .tools
        .sort_by(|left, right| left.name.cmp(&right.name));
    let tools = catalog.tools.clone();
    CACHE_STATE
        .lock()
        .map_err(|_| "registre MCP indisponible")?
        .publish(&connector.id, connector.generation, catalog, Instant::now())?;
    Ok(tools)
}

pub async fn resolve_enabled_tool(
    connector_id: &str,
    tool_name: &str,
) -> Result<(EnabledConnector, McpToolDef), String> {
    config::validate_connector_id(connector_id)?;
    let connector = get_enabled_connectors()?
        .into_iter()
        .find(|connector| connector.id == connector_id)
        .ok_or_else(|| "outil MCP indisponible".to_string())?;
    let tools = get_tools(&connector).await?;
    let tool = with_current_generation(connector.generation, || {
        let active = config::find(connector_id)?
            .is_some_and(|stored| stored.status == "connected" && stored.enabled_in_chat);
        if !active {
            return Err("outil MCP indisponible".to_string());
        }
        select_exact_tool(&tools, tool_name)
    })?;
    Ok((connector, tool))
}

pub async fn call_enabled_tool(
    connector: &EnabledConnector,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<McpToolResult, McpCallError> {
    authorize_business_send(&connector.id, connector.generation)?;
    connector.transport.call_tool(tool_name, arguments).await
}

pub(crate) fn authorize_business_send(
    connector_id: &str,
    generation: u64,
) -> Result<(), McpCallError> {
    with_current_generation(generation, || {
        let active = config::find(connector_id)?
            .is_some_and(|stored| stored.status == "connected" && stored.enabled_in_chat);
        if active {
            Ok(())
        } else {
            Err("outil MCP indisponible".to_string())
        }
    })
    .map_err(|_| McpCallError::Unavailable)
}

pub(crate) fn select_exact_tool(
    tools: &[McpToolDef],
    tool_name: &str,
) -> Result<McpToolDef, String> {
    let mut matches = tools.iter().filter(|tool| tool.name == tool_name);
    let tool = matches
        .next()
        .ok_or_else(|| "outil MCP indisponible".to_string())?;
    if matches.next().is_some() {
        return Err("catalogue MCP invalide".to_string());
    }
    Ok(tool.clone())
}

include!("registry_probe.rs");

pub fn current_generation() -> Result<u64, String> {
    let state = CACHE_STATE
        .lock()
        .map_err(|_| "registre MCP indisponible")?;
    if state.is_closed() {
        return Err("registre MCP indisponible".to_string());
    }
    Ok(state.generation())
}

pub fn with_current_generation<T>(
    generation: u64,
    action: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let state = CACHE_STATE
        .lock()
        .map_err(|_| "registre MCP indisponible")?;
    if state.is_closed() || state.generation() != generation {
        return Err("identité MCP modifiée".to_string());
    }
    action()
}

pub(crate) fn mutate_identity<T>(
    connector_id: &str,
    action: impl FnOnce(&IdentityMutation) -> Result<T, String>,
) -> Result<T, String> {
    mutate_identity_checked(connector_id, None, action)
}

pub(crate) fn mutate_identity_if_generation<T>(
    connector_id: &str,
    expected_generation: u64,
    action: impl FnOnce(&IdentityMutation) -> Result<T, String>,
) -> Result<T, String> {
    mutate_identity_checked(connector_id, Some(expected_generation), action)
}

fn mutate_identity_checked<T>(
    connector_id: &str,
    expected_generation: Option<u64>,
    action: impl FnOnce(&IdentityMutation) -> Result<T, String>,
) -> Result<T, String> {
    config::validate_connector_id(connector_id)?;
    let mut state = CACHE_STATE
        .lock()
        .map_err(|_| "registre MCP indisponible")?;
    if state.is_closed()
        || expected_generation.is_some_and(|expected| state.generation() != expected)
    {
        return Err("registre MCP indisponible".to_string());
    }
    state.invalidate(connector_id);
    action(&IdentityMutation(()))
}
