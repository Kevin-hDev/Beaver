use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::http::HttpTransport;
use super::stdio::StdioTransport;
use super::transport::{McpToolDef, McpTransport};
use super::{config, process_manager, token_validation, trusted};

const MAX_CACHE: usize = 32;
const CACHE_TTL_SECS: u64 = 300;
const TEST_TIMEOUT_SECS: u64 = 20;

struct CachedTools {
    tools: Vec<McpToolDef>,
    fetched_at: Instant,
}

static TOOL_CACHE: std::sync::LazyLock<Mutex<HashMap<String, CachedTools>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct EnabledConnector {
    pub id: String,
    pub transport: Arc<dyn McpTransport>,
}

pub fn get_enabled_connectors() -> Result<Vec<EnabledConnector>, String> {
    Ok(config::load()?
        .into_iter()
        .filter(|c| c.status == "connected" && c.enabled_in_chat)
        .filter_map(build_connector)
        .take(config::MAX_CONNECTORS)
        .collect())
}

pub fn is_trusted_endpoint_pub(connector_id: &str, url: &str) -> bool {
    trusted::is_trusted_endpoint_for_connector(connector_id, url)
}

fn build_connector(c: config::StoredConnector) -> Option<EnabledConnector> {
    if !config::is_valid_connector_id(&c.id) {
        return None;
    }
    if c.id == "imessage" && !cfg!(target_os = "macos") {
        return None;
    }
    if let Some(ref endpoint) = c.endpoint {
        if trusted::is_trusted_endpoint_for_connector(&c.id, endpoint) {
            let transport = HttpTransport::new(c.id.clone(), endpoint.clone());
            return Some(EnabledConnector {
                id: c.id,
                transport: Arc::new(transport),
            });
        }
    }

    if let Some(cmd) = config::install_command_for(&c) {
        let env_key_names = config::validated_env_keys(c.env_keys.as_deref()).ok()?;
        let transport = StdioTransport::new(c.id.clone(), cmd, env_key_names);
        return Some(EnabledConnector {
            id: c.id,
            transport: Arc::new(transport),
        });
    }

    None
}

pub async fn get_tools(connector: &EnabledConnector) -> Result<Vec<McpToolDef>, String> {
    if let Some(cached) = get_cached(&connector.id) {
        return Ok(cached);
    }

    let tools = connector.transport.list_tools().await?;
    set_cached(&connector.id, &tools);
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
    let active = config::find(connector_id)?
        .is_some_and(|stored| stored.status == "connected" && stored.enabled_in_chat);
    if !active {
        return Err("outil MCP indisponible".to_string());
    }
    let tool = select_exact_tool(&tools, tool_name)?;
    Ok((connector, tool))
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

pub fn invalidate_cache(connector_id: &str) {
    if let Ok(mut cache) = TOOL_CACHE.lock() {
        cache.remove(connector_id);
    }
}

fn get_cached(connector_id: &str) -> Option<Vec<McpToolDef>> {
    let cache = TOOL_CACHE.lock().ok()?;
    let entry = cache.get(connector_id)?;
    if entry.fetched_at.elapsed().as_secs() > CACHE_TTL_SECS {
        return None;
    }
    Some(entry.tools.clone())
}

fn set_cached(connector_id: &str, tools: &[McpToolDef]) {
    if let Ok(mut cache) = TOOL_CACHE.lock() {
        if cache.len() >= MAX_CACHE && !cache.contains_key(connector_id) {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.fetched_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }
        cache.insert(
            connector_id.to_string(),
            CachedTools {
                tools: tools.to_vec(),
                fetched_at: Instant::now(),
            },
        );
    }
}
