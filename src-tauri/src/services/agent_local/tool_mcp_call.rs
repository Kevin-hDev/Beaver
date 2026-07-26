use serde_json::Value;
use std::time::Duration;

use crate::services::agent_local::types_tools::ToolResult;
use crate::services::mcp_bridge::{arguments, config, registry};

const MCP_CALL_TIMEOUT: Duration = Duration::from_secs(60);

pub(super) async fn call(args: &Value) -> ToolResult {
    let Some(tool_id) = args.get("tool_id").and_then(Value::as_str) else {
        return ToolResult::err("outil MCP invalide".to_string());
    };
    let Some((connector_id, tool_name)) = tool_id.split_once('.') else {
        return ToolResult::err("outil MCP invalide".to_string());
    };
    if config::validate_connector_id(connector_id).is_err() || !valid_tool_name(tool_name) {
        return ToolResult::err("outil MCP invalide".to_string());
    }

    let empty_arguments = Value::Object(Default::default());
    let arguments = args.get("arguments").unwrap_or(&empty_arguments);
    let (connector, tool) = match registry::resolve_enabled_tool(connector_id, tool_name).await {
        Ok(resolved) => resolved,
        Err(_) => return ToolResult::err("outil MCP indisponible".to_string()),
    };
    if arguments::validate(arguments, tool.input_schema.as_ref()).is_err() {
        return ToolResult::err("arguments MCP invalides".to_string());
    }

    match tokio::time::timeout(
        MCP_CALL_TIMEOUT,
        connector.transport.call_tool(&tool.name, arguments.clone()),
    )
    .await
    {
        Ok(Ok(result)) => ToolResult::ok(sanitize_output(&result)),
        Ok(Err(_)) => ToolResult::err("appel MCP échoué".to_string()),
        Err(_) => ToolResult::err("appel MCP expiré".to_string()),
    }
}

fn valid_tool_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn sanitize_output(output: &str) -> String {
    output
        .chars()
        .take(4096)
        .filter(|character| {
            (!character.is_control() || matches!(character, '\n' | '\t'))
                && !matches!(character, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}')
        })
        .collect()
}
