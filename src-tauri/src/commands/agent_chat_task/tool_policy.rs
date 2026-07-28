use serde_json::Value;

pub const GROQ_EXTENSIONS_NOTICE: &str = "agentLocal.groqExtensionsUnavailable";

pub struct ToolPolicy {
    pub tools: Vec<Value>,
    pub extensions_blocked: bool,
}

pub fn apply(provider_id: &str, model: &str, tools: Vec<Value>) -> ToolPolicy {
    apply_with(provider_id, model, tools, |name| {
        crate::services::extensions::is_dynamic_tool(name)
    })
}

fn apply_with(
    provider_id: &str,
    model: &str,
    mut tools: Vec<Value>,
    is_extension: impl Fn(&str) -> bool,
) -> ToolPolicy {
    if allows_extension_tools(provider_id, model) {
        return ToolPolicy {
            tools,
            extensions_blocked: false,
        };
    }
    let extensions_blocked = tools
        .iter()
        .any(|tool| is_extension_tool(tool, &is_extension));
    if is_groq_compound(provider_id, model) {
        tools.clear();
        return ToolPolicy {
            tools,
            extensions_blocked,
        };
    }
    tools.retain(|tool| {
        tool.pointer("/function/name")
            .and_then(Value::as_str)
            .is_some_and(|name| {
                name != crate::services::extensions::SEARCH_TOOL_NAME
                    && keep_tool(provider_id, model, is_extension(name))
            })
    });
    ToolPolicy {
        extensions_blocked,
        tools,
    }
}

fn is_extension_tool(tool: &Value, is_extension: &impl Fn(&str) -> bool) -> bool {
    tool.pointer("/function/name")
        .and_then(Value::as_str)
        .is_some_and(is_extension)
}

fn allows_extension_tools(provider_id: &str, model: &str) -> bool {
    !is_groq_family(provider_id, model)
}

fn keep_tool(provider_id: &str, model: &str, extension_tool: bool) -> bool {
    !extension_tool || allows_extension_tools(provider_id, model)
}

fn is_groq_compound(provider_id: &str, model: &str) -> bool {
    is_groq_family(provider_id, model)
        && model
            .rsplit_once('/')
            .map_or(model, |(_, name)| name)
            .to_ascii_lowercase()
            .starts_with("compound")
}

fn is_groq_family(provider_id: &str, model: &str) -> bool {
    provider_id == "groq"
        || (provider_id == "openrouter" && model.to_ascii_lowercase().starts_with("groq/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_groq_blocks_extension_tools() {
        assert!(!allows_extension_tools("groq", "llama-3.3"));
        assert!(!allows_extension_tools("openrouter", "groq/llama-3.3-70b"));
        assert!(allows_extension_tools("openai", "gpt-5.6"));
        assert!(allows_extension_tools("moonshot", "kimi-k2.7"));
        assert!(!keep_tool("groq", "llama-3.3", true));
        assert!(keep_tool("groq", "llama-3.3", false));
        assert!(is_groq_compound("groq", "groq/compound"));
        assert!(is_groq_compound("openrouter", "groq/compound"));
        assert!(is_groq_compound("groq", "compound-mini"));
        assert!(!is_groq_compound("groq", "llama-3.3-70b-versatile"));
    }

    #[test]
    fn groq_keeps_core_tools_and_removes_complete_plugins() {
        let tools = vec![
            serde_json::json!({"function": {"name": "read_file"}}),
            serde_json::json!({"function": {"name": "search_extension_tools"}}),
            serde_json::json!({"function": {"name": "beaver.office.documents.create"}}),
        ];

        let policy = apply_with("groq", "llama-3.3-70b-versatile", tools, |name| {
            name.starts_with("beaver.office.")
        });

        assert!(policy.extensions_blocked);
        assert_eq!(policy.tools.len(), 1);
        assert_eq!(policy.tools[0]["function"]["name"], "read_file");
    }

    #[test]
    fn groq_compound_receives_no_tools() {
        let tools = vec![serde_json::json!({"function": {"name": "read_file"}})];

        let policy = apply_with("groq", "groq/compound", tools, |_| false);

        assert!(policy.tools.is_empty());
    }
}
