use std::collections::HashSet;
use std::time::Duration;

use rmcp::model::{CacheScope, ClientConfig, PaginatedRequestParams, ProtocolVersion};
use rmcp::service::RunningService;
use rmcp::RoleClient;

use super::transport::{McpToolCatalog, McpToolDef, MAX_TOOLS};

fn ttl(
    version: &ProtocolVersion,
    value: Option<u64>,
    scope: Option<CacheScope>,
) -> Option<Duration> {
    if version < &ProtocolVersion::V_2026_07_28 {
        return Some(super::registry_cache::FALLBACK_TTL);
    }
    match (value, scope) {
        (Some(ms), Some(CacheScope::Public | CacheScope::Private)) if ms > 0 => Some(
            Duration::from_millis(ms.min(super::registry_cache::FALLBACK_TTL.as_millis() as u64)),
        ),
        _ => None,
    }
}

pub(super) async fn list(
    service: &RunningService<RoleClient, ClientConfig>,
) -> Result<McpToolCatalog, String> {
    let version = service
        .peer_info()
        .ok_or("connexion MCP invalide")?
        .protocol_version
        .clone();
    let mut cursor = None;
    let mut seen = HashSet::new();
    let mut tools = Vec::new();
    let mut cache_ttl = Some(super::registry_cache::FALLBACK_TTL);
    for _ in 0..MAX_TOOLS {
        let params = cursor
            .take()
            .map(|cursor| PaginatedRequestParams::default().with_cursor(Some(cursor)));
        let page = service
            .list_tools(params)
            .await
            .map_err(|_| "catalogue MCP indisponible")?;
        if tools
            .len()
            .checked_add(page.tools.len())
            .is_none_or(|count| count > MAX_TOOLS)
        {
            return Err("catalogue MCP invalide".to_string());
        }
        let page_ttl = ttl(&version, page.ttl_ms, page.cache_scope);
        cache_ttl = cache_ttl.zip(page_ttl).map(|(left, right)| left.min(right));
        for tool in page.tools {
            let value = serde_json::to_value(tool).map_err(|_| "catalogue MCP invalide")?;
            tools.push(
                serde_json::from_value::<McpToolDef>(value)
                    .map_err(|_| "catalogue MCP invalide")?,
            );
        }
        match page.next_cursor {
            Some(next) if !next.is_empty() && next.len() <= 256 && seen.insert(next.clone()) => {
                cursor = Some(next);
            }
            None => {
                return Ok(McpToolCatalog {
                    tools: super::transport::validate_tools(tools)?,
                    cache_ttl,
                })
            }
            _ => return Err("pagination MCP invalide".to_string()),
        }
    }
    Err("pagination MCP trop longue".to_string())
}
