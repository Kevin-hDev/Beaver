use serde_json::Value;

use super::extension_session_state::ExtensionSessionState;
use super::extension_tool_selection::{
    decide_for_catalog, PluginDescriptor,
};
use super::types_tools::ToolResult;

#[derive(Debug, PartialEq, Eq)]
enum DiscoveryStatus {
    Loaded,
    AlreadyAvailable,
    NoTools,
    ProviderLimit,
    DiscoveryLimit,
    Unavailable,
}

struct DiscoveryLine {
    plugin_name: String,
    status: DiscoveryStatus,
}

pub async fn execute(args: &Value, session_id: &str) -> ToolResult {
    let Some(query) = args.get("query").and_then(Value::as_str) else {
        return ToolResult::validation(
            "plugin_search_query_invalid",
            "Recherche de plugins invalide.",
        );
    };
    if query.chars().count() > crate::services::extensions::MAX_SEARCH_QUERY_CHARS {
        return ToolResult::validation(
            "plugin_search_query_invalid",
            "Recherche de plugins invalide.",
        );
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
        Ok(lines) => discovery_result(lines),
        Err(_) => ToolResult::unavailable(
            "plugin_search_unavailable",
            "Recherche de plugins indisponible.",
            true,
        ),
    }
}

fn discovery_result(lines: Vec<DiscoveryLine>) -> ToolResult {
    let incomplete = lines.iter().any(|line| {
        matches!(
            line.status,
            DiscoveryStatus::ProviderLimit
                | DiscoveryStatus::DiscoveryLimit
                | DiscoveryStatus::Unavailable
        )
    });
    let output = render(lines);
    if incomplete {
        ToolResult::partial(
            output,
            ["Certains outils correspondants n'ont pas pu être chargés."],
        )
    } else {
        ToolResult::ok(output)
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
    let plugins = state.plugin_descriptors.clone();
    let catalog = crate::services::extensions::catalog_snapshot();
    let mut discovered = state.discovered_plugin_ids.clone();
    let mut lines = Vec::with_capacity(matches.len());
    for candidate in matches {
        let current = selection(&plugins, &catalog, masked, state.plugin_tool_capacity, &discovered);
        let descriptor = plugins
            .iter()
            .find(|plugin| plugin.id == candidate.extension_id);
        let Some(descriptor) = descriptor else {
            lines.push(DiscoveryLine {
                plugin_name: candidate.extension_name.clone(),
                status: DiscoveryStatus::Unavailable,
            });
            continue;
        };
        let status = if let Some(status) = existing_status(
            descriptor.definition_count,
            current.active_plugin_ids.contains(&candidate.extension_id),
        ) {
            if status == DiscoveryStatus::NoTools
                && !push_unique(&mut discovered, &candidate.extension_id)
            {
                lines.push(DiscoveryLine {
                    plugin_name: candidate.extension_name.clone(),
                    status: DiscoveryStatus::DiscoveryLimit,
                });
                continue;
            }
            status
        } else {
            let mut proposed = discovered.clone();
            if !push_unique(&mut proposed, &candidate.extension_id) {
                lines.push(DiscoveryLine {
                    plugin_name: candidate.extension_name.clone(),
                    status: DiscoveryStatus::DiscoveryLimit,
                });
                continue;
            }
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
    state.active_plugin_ids = selection(
        &plugins,
        &catalog,
        masked,
        state.plugin_tool_capacity,
        &discovered,
    )
    .active_plugin_ids;
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
    decide_for_catalog(
        plugins,
        catalog,
        masked,
        tool_capacity,
        discovered_plugin_ids,
    )
}

fn push_unique(values: &mut Vec<String>, value: &str) -> bool {
    if values.iter().any(|current| current == value) {
        return true;
    }
    if values.len() >= crate::services::extensions::MAX_DISCOVERED_PLUGINS {
        return false;
    }
    values.push(value.to_string());
    true
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
            DiscoveryStatus::DiscoveryLimit => format!(
                "- {} : non chargé, car la limite de plugins découverts pour cette session est atteinte.",
                line.plugin_name
            ),
            DiscoveryStatus::Unavailable => format!(
                "- {} : outils indisponibles dans cette requête.",
                line.plugin_name
            ),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "tool_extension_discovery_tests.rs"]
mod tests;
