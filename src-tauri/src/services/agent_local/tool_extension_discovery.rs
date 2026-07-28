use serde_json::Value;

use super::extension_session_state::ExtensionSessionState;
use super::extension_tool_selection::{
    decide, PluginDescriptor, SelectionPolicy,
};
use super::types_tools::ToolResult;

#[derive(Debug, PartialEq, Eq)]
enum DiscoveryStatus {
    Loaded,
    AlreadyAvailable,
    NoTools,
    ProviderLimit,
}

struct DiscoveryLine {
    plugin_name: String,
    status: DiscoveryStatus,
}

pub async fn execute(args: &Value, session_id: &str) -> ToolResult {
    let Some(query) = args.get("query").and_then(Value::as_str) else {
        return ToolResult::err("Recherche de plugins invalide.");
    };
    if query.chars().count() > crate::services::extensions::MAX_SEARCH_QUERY_CHARS {
        return ToolResult::err("Recherche de plugins invalide.");
    }
    let matches = crate::services::extensions::search_plugins(
        query,
        crate::services::extensions::MAX_SEARCH_RESULTS,
    );
    if matches.is_empty() {
        return ToolResult::ok("Aucun plugin activé ne correspond à cette recherche.");
    }
    let result = super::extension_session_state::mutate(session_id, |state| {
        discover_matches(state, &matches)
    })
    .await;
    match result {
        Ok(lines) => ToolResult::ok(render(lines)),
        Err(_) => ToolResult::err("Recherche de plugins indisponible."),
    }
}

fn discover_matches(
    state: &mut ExtensionSessionState,
    matches: &[crate::services::extensions::PluginMatch],
) -> Result<Vec<DiscoveryLine>, String> {
    let masked = state
        .epoch
        .as_ref()
        .ok_or_else(|| "État de découverte absent.".to_string())?
        .masked;
    let plugins = descriptors();
    let catalog = crate::services::extensions::catalog_snapshot();
    let mut discovered = state.discovered_plugin_ids.clone();
    let mut lines = Vec::with_capacity(matches.len());
    for candidate in matches {
        let current = selection(&plugins, &catalog, masked, state.plugin_tool_capacity, &discovered);
        let tool_count = plugins
            .iter()
            .find(|plugin| plugin.id == candidate.extension_id)
            .map(|plugin| plugin.tool_count)
            .unwrap_or_default();
        let status = if let Some(status) = existing_status(
            tool_count,
            current.active_plugin_ids.contains(&candidate.extension_id),
        ) {
            if status == DiscoveryStatus::NoTools {
                push_unique(&mut discovered, &candidate.extension_id);
            }
            status
        } else {
            let mut proposed = discovered.clone();
            push_unique(&mut proposed, &candidate.extension_id);
            let next = selection(
                &plugins,
                &catalog,
                masked,
                state.plugin_tool_capacity,
                &proposed,
            );
            if next.active_plugin_ids.contains(&candidate.extension_id) {
                discovered = proposed;
                DiscoveryStatus::Loaded
            } else {
                DiscoveryStatus::ProviderLimit
            }
        };
        lines.push(DiscoveryLine {
            plugin_name: candidate.extension_name.clone(),
            status,
        });
    }
    state.discovered_plugin_ids = discovered;
    Ok(lines)
}

fn existing_status(tool_count: usize, active: bool) -> Option<DiscoveryStatus> {
    if tool_count == 0 {
        Some(DiscoveryStatus::NoTools)
    } else if active {
        Some(DiscoveryStatus::AlreadyAvailable)
    } else {
        None
    }
}

fn selection(
    plugins: &[PluginDescriptor],
    catalog: &crate::services::extensions::CatalogSnapshot,
    masked: bool,
    tool_capacity: usize,
    discovered_plugin_ids: &[String],
) -> super::extension_tool_selection::CapacityDecision {
    decide(
        plugins,
        SelectionPolicy {
            masked,
            tool_capacity,
            ordered_plugin_ids: &catalog.ordered_plugin_ids,
            protected_plugin_ids: &catalog.protected_plugin_ids,
            essential_plugin_ids: &catalog.essential_plugin_ids,
            discovered_plugin_ids,
        },
    )
}

fn descriptors() -> Vec<PluginDescriptor> {
    crate::services::extensions::indexed_plugins()
        .into_iter()
        .map(|plugin| PluginDescriptor {
            id: plugin.id,
            tool_count: plugin.tools.len(),
            replaces_core: plugin.tools.iter().any(|tool| tool.replaces_core),
        })
        .collect()
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if values.len() < crate::services::extensions::MAX_DISCOVERED_PLUGINS
        && !values.iter().any(|current| current == value)
    {
        values.push(value.to_string());
    }
}

fn render(lines: Vec<DiscoveryLine>) -> String {
    lines
        .into_iter()
        .map(|line| match line.status {
            DiscoveryStatus::Loaded => {
                format!("- {} : outils chargés pour le prochain tour.", line.plugin_name)
            }
            DiscoveryStatus::AlreadyAvailable => {
                format!("- {} : outils déjà disponibles.", line.plugin_name)
            }
            DiscoveryStatus::NoTools => format!(
                "- {} : plugin actif, sans outil appelable.",
                line.plugin_name
            ),
            DiscoveryStatus::ProviderLimit => format!(
                "- {} : non chargé, car le plafond d'outils du fournisseur serait dépassé.",
                line.plugin_name
            ),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "tool_extension_discovery_tests.rs"]
mod tests;
