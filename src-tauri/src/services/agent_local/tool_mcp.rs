use serde_json::Value;

use crate::services::agent_local::types_tools::ToolResult;
use crate::services::mcp_bridge::registry;
use crate::services::mcp_bridge::transport::McpToolDef;

const MAX_TOOLS_PER_SERVICE: usize = 15;

pub async fn execute(args: &Value) -> ToolResult {
    let mode = args["mode"].as_str().unwrap_or("search");
    match mode {
        "search" => search(args).await,
        "call" => super::tool_mcp_call::call(args).await,
        _ => ToolResult::err("mode invalide : utiliser 'search' ou 'call'".to_string()),
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
        Err(_) => return ToolResult::err("configuration MCP indisponible".to_string()),
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

    let mut output = String::new();

    if !sections.is_empty() {
        let total: usize = sections.iter().map(|s| s.matches("\n  - ").count()).sum();
        output.push_str(&format!(
            "{total} outils MCP trouvés :\n\n{}",
            sections.join("\n\n")
        ));
    }

    if !errors.is_empty() {
        if !output.is_empty() {
            output.push_str("\n\n");
        }
        output.push_str(&format!("Erreurs :\n{}", errors.join("\n")));
    }

    if output.is_empty() {
        return ToolResult::ok("Aucun outil MCP ne correspond à la recherche.".to_string());
    }

    ToolResult::ok(output)
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
