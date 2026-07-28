use serde_json::Value;

use super::types_tools::ToolResult;

pub fn execute(args: &Value) -> ToolResult {
    let Some(query) = args.get("query").and_then(Value::as_str) else {
        return ToolResult::err("Recherche d'extensions invalide.");
    };
    if query.chars().count() > crate::services::extensions::MAX_SEARCH_QUERY_CHARS {
        return ToolResult::err("Recherche d'extensions invalide.");
    }
    let matches = crate::services::extensions::search_tools(
        query,
        crate::services::extensions::MAX_SEARCH_RESULTS,
    );
    if matches.is_empty() {
        return ToolResult::ok("No matching extension tools were found.");
    }
    let lines = matches
        .iter()
        .map(|item| {
            format!(
                "- {} — {} ({})",
                item.tool_name, item.description, item.extension_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    ToolResult::ok(format!(
        "Matching extension tools are available for the next model turn:\n{lines}"
    ))
}
