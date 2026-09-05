use serde_json::Value;

pub const GROQ_EXTENSIONS_NOTICE: &str = "agentLocal.groqExtensionsUnavailable";

pub struct ToolPolicy {
    pub tools: Vec<Value>,
    pub extensions_blocked: bool,
}

pub fn apply(
    policy: crate::services::llm::route_profile::ExtensionToolPolicy,
    tools: Vec<Value>,
) -> ToolPolicy {
    apply_with(policy, tools, |name| {
        crate::services::extensions::is_dynamic_tool(name)
    })
}

fn apply_with(
    policy: crate::services::llm::route_profile::ExtensionToolPolicy,
    mut tools: Vec<Value>,
    is_extension: impl Fn(&str) -> bool,
) -> ToolPolicy {
    use crate::services::llm::route_profile::ExtensionToolPolicy;

    if policy == ExtensionToolPolicy::All {
        return ToolPolicy {
            tools,
            extensions_blocked: false,
        };
    }
    let extensions_blocked = tools
        .iter()
        .any(|tool| is_extension_tool(tool, &is_extension));
    if policy == ExtensionToolPolicy::NoTools {
        tools.clear();
        return ToolPolicy {
            tools,
            extensions_blocked,
        };
    }
    tools = tools
        .into_iter()
        .filter_map(|tool| {
            let name = tool.pointer("/function/name").and_then(Value::as_str)?;
            if name == crate::services::extensions::LIST_EXTENSIONS_TOOL_NAME
                || name == crate::services::extensions::INSPECT_EXTENSIONS_TOOL_NAME
                || name == crate::services::agent_local::tool_extension_resource::NAME
            {
                return None;
            }
            if !is_extension(name) {
                Some(tool)
            } else {
                crate::services::extensions::core_fallback(&tool).cloned()
            }
        })
        .collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::llm::route_profile::ExtensionToolPolicy;

    #[test]
    fn openrouter_groq_models_block_extension_tools() {
        assert_eq!(
            crate::services::llm::route_profile::tool_policy("openrouter", "groq/llama-3.3-70b")
                .unwrap()
                .extensions,
            ExtensionToolPolicy::WithoutExtensions
        );
        assert_eq!(
            crate::services::llm::route_profile::tool_policy("openrouter", "groq/compound")
                .unwrap()
                .extensions,
            ExtensionToolPolicy::NoTools
        );
    }

    #[test]
    fn openrouter_groq_keeps_core_tools_and_removes_complete_plugins() {
        let tools = vec![
            serde_json::json!({"function": {"name": "read_file"}}),
            serde_json::json!({"function": {"name": "list_extensions"}}),
            serde_json::json!({"function": {"name": "inspect_extensions"}}),
            serde_json::json!({"function": {"name": "load_extension_resource"}}),
            serde_json::json!({"function": {"name": "beaver.office.documents.create"}}),
        ];

        let policy = apply_with(ExtensionToolPolicy::WithoutExtensions, tools, |name| {
            name.starts_with("beaver.office.")
        });

        assert!(policy.extensions_blocked);
        assert_eq!(policy.tools.len(), 1);
        assert_eq!(policy.tools[0]["function"]["name"], "read_file");
    }

    #[test]
    fn openrouter_groq_compound_receives_no_tools() {
        let tools = vec![serde_json::json!({"function": {"name": "read_file"}})];

        let policy = apply_with(ExtensionToolPolicy::NoTools, tools, |_| false);

        assert!(policy.tools.is_empty());
    }

    #[test]
    fn openrouter_groq_restores_native_tools_hidden_by_plugin_replacements() {
        let tools = vec![serde_json::json!({
            "_beaverCoreFallback": {
                "function": {"name": "read_file", "description": "native"}
            },
            "function": {"name": "read_file", "description": "plugin"}
        })];

        let policy = apply_with(ExtensionToolPolicy::WithoutExtensions, tools, |_| true);

        assert_eq!(policy.tools.len(), 1);
        assert_eq!(policy.tools[0]["function"]["description"], "native");
    }
}
