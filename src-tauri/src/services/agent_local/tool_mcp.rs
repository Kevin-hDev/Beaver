use serde_json::Value;

use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::mcp_bridge::registry;
use crate::services::mcp_bridge::transport::McpToolDef;

const MAX_TOOLS_PER_SERVICE: usize = 15;

pub async fn execute(args: &Value) -> ToolResult {
    let mode = args["mode"].as_str().unwrap_or("search");
    match mode {
        "search" => search(args).await,
        "call" => super::tool_mcp_call::call(args).await,
        _ => ToolResult::error(
            "mode invalide : utiliser 'search' ou 'call'",
            "invalid_mcp_mode",
            ToolErrorCategory::Validation,
            false,
        ),
    }
}

async fn search(args: &Value) -> ToolResult {
    let raw_query = args["query"].as_str().unwrap_or("").to_lowercase();
    let keywords: Vec<&str> = raw_query
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();
    let connectors = match registry::get_enabled_connectors() {
        Ok(connectors) => connectors,
        Err(_) => {
            return ToolResult::error(
                "configuration MCP indisponible",
                "mcp_configuration_unavailable",
                ToolErrorCategory::Internal,
                true,
            )
        }
    };

    if connectors.is_empty() {
        return ToolResult::ok("Aucun connecteur MCP activé.".to_string());
    }

    let mut sections: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for connector in &connectors {
        let tools = match registry::get_tools(connector).await {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("{}: {e}", connector.id));
                continue;
            }
        };

        let matched: Vec<String> = tools
            .iter()
            .filter(|t| matches_keywords(t, &keywords, &connector.id))
            .take(MAX_TOOLS_PER_SERVICE)
            .map(|t| {
                let tool_id = format!("{}.{}", connector.id, t.name);
                let desc = t.description.as_deref().unwrap_or("(pas de description)");
                format!("  - {tool_id} : {desc}")
            })
            .collect();

        if !matched.is_empty() {
            sections.push(format!(
                "**{}** ({} outils) :\n{}",
                connector.id,
                matched.len(),
                matched.join("\n")
            ));
        }
    }

    search_result(sections, errors)
}

fn search_result(sections: Vec<String>, errors: Vec<String>) -> ToolResult {
    if sections.is_empty() && errors.is_empty() {
        return ToolResult::ok("Aucun outil MCP ne correspond à la recherche.");
    }
    if sections.is_empty() {
        return ToolResult::error(
            format!("Catalogue MCP indisponible :\n{}", errors.join("\n")),
            "mcp_catalog_unavailable",
            ToolErrorCategory::External,
            true,
        );
    }

    let total: usize = sections.iter().map(|s| s.matches("\n  - ").count()).sum();
    let output = format!(
        "{total} outils MCP trouvés :\n\n{}",
        sections.join("\n\n")
    );
    if errors.is_empty() {
        ToolResult::ok(output)
    } else {
        ToolResult::partial(
            output,
            errors
                .into_iter()
                .map(|error| format!("Connecteur MCP ignoré : {error}")),
        )
    }
}

fn matches_keywords(tool: &McpToolDef, keywords: &[&str], connector_id: &str) -> bool {
    if keywords.is_empty() {
        return true;
    }
    let name = tool.name.to_lowercase();
    let desc = tool.description.as_deref().unwrap_or("").to_lowercase();
    let cid = connector_id.to_lowercase();
    keywords
        .iter()
        .any(|kw| name.contains(kw) || desc.contains(kw) || cid.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::search_result;
    use crate::services::agent_local::tool_result_contract::ToolResultStatus;

    #[test]
    fn connector_failures_are_never_reported_as_clean_successes() {
        let failed = search_result(Vec::new(), vec!["calendar: timeout".into()]);
        let partial = search_result(
            vec!["**files** (1 outils) :\n  - files.read : read".into()],
            vec!["calendar: timeout".into()],
        );

        assert_eq!(failed.status, ToolResultStatus::Error);
        assert_eq!(failed.error.unwrap().code.as_ref(), "mcp_catalog_unavailable");
        assert_eq!(partial.status, ToolResultStatus::Partial);
        assert_eq!(partial.warnings.len(), 1);
    }
}
