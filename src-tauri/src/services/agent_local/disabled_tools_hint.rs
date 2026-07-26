//! Builds the "Disabled tools" hint injected into the system prompt so the
//! LLM knows which optional tools exist but are currently disabled in settings.

use super::types_ollama::ChatMessage;

/// Append a "Disabled tools" section to the system prompt when at least one
/// optional tool is disabled. The hint lists each disabled tool id with a
/// one-line description (`tool_short_desc`) and instructs the model NOT to
/// call them — the runtime guard in `tool_dispatcher` already blocks any
/// attempt with an explicit error.
pub fn prepend(messages: &mut [ChatMessage], enabled_tool_names: &[String]) {
    let disabled: Vec<&'static str> = super::tool_catalog::catalog()
        .iter()
        .filter(|t| !t.locked)
        .filter(|t| !enabled_tool_names.iter().any(|e| e == t.id))
        .map(|t| t.id)
        .collect();
    if disabled.is_empty() {
        return;
    }
    let listing = disabled_listing(&disabled);
    if listing.is_empty() {
        return;
    }
    let section = format!(
        "\n\n## Disabled tools\n\
         The following tools exist but are DISABLED in settings. Do NOT attempt to call them — they will fail. \
         If the user's task would benefit from one, tell them they can enable it in Settings → Tools.\n\
         {listing}"
    );
    if let Some(first) = messages.first_mut().filter(|m| m.role == "system") {
        first.content.push_str(&section);
    }
}

fn disabled_listing(disabled: &[&str]) -> String {
    let subagents_disabled = disabled.contains(&"delegate_task");
    let mut lines = Vec::new();
    if subagents_disabled {
        lines.push(
            "- subagents: Spawn and control child agents for parallel or isolated work".to_string(),
        );
    }
    lines.extend(
        disabled
            .iter()
            .filter(|id| !(subagents_disabled && super::tool_catalog::is_subagent_tool(id)))
            .filter_map(|id| {
                super::tool_short_desc::tool_short_desc(id).map(|d| format!("- {id}: {d}"))
            }),
    );
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    #[test]
    fn subagents_are_collapsed_as_one_disabled_feature() {
        let listing = super::disabled_listing(&[
            "delegate_task",
            "list_subagents",
            "get_subagent",
            "archive_subagent",
            "forecast",
        ]);

        assert!(listing.contains("- subagents:"));
        assert!(!listing.contains("- delegate_task:"));
        assert!(!listing.contains("- list_subagents:"));
        assert!(!listing.contains("- archive_subagent:"));
        assert!(listing.contains("- forecast:"));
    }
}
