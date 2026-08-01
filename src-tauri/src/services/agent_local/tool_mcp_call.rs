use serde_json::Value;
use std::time::Duration;

use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::mcp_bridge::{arguments, config, registry};

const MCP_CALL_TIMEOUT: Duration = Duration::from_secs(60);

pub(super) async fn call(args: &Value) -> ToolResult {
    let Some(tool_id) = args.get("tool_id").and_then(Value::as_str) else {
        return invalid_tool();
    };
    let Some((connector_id, tool_name)) = tool_id.split_once('.') else {
        return invalid_tool();
    };
    if config::validate_connector_id(connector_id).is_err() || !valid_tool_name(tool_name) {
        return invalid_tool();
    }

    let empty_arguments = Value::Object(Default::default());
    let arguments = args.get("arguments").unwrap_or(&empty_arguments);
    let (connector, tool) = match registry::resolve_enabled_tool(connector_id, tool_name).await {
        Ok(resolved) => resolved,
        Err(_) => {
            return ToolResult::error(
                "outil MCP indisponible",
                "mcp_tool_unavailable",
                ToolErrorCategory::Unavailable,
                true,
            )
        }
    };
    if arguments::validate(arguments, tool.input_schema.as_ref()).is_err() {
        return ToolResult::error(
            "arguments MCP invalides",
            "invalid_mcp_arguments",
            ToolErrorCategory::Validation,
            false,
        );
    }

    match tokio::time::timeout(
        MCP_CALL_TIMEOUT,
        connector.transport.call_tool(&tool.name, arguments.clone()),
    )
    .await
    {
        Ok(Ok(result)) => to_tool_result(result),
        Ok(Err(error)) => transport_failure(error),
        Err(_) => ToolResult::timeout(
            "mcp_call_timeout",
            "appel MCP expiré",
            false,
        )
        .with_error_hint(
            "Vérifier l'état du service avant de relancer : l'action a pu être exécutée.",
        ),
    }
}

fn transport_failure(error: crate::services::mcp_bridge::transport::McpCallError) -> ToolResult {
    use crate::services::mcp_bridge::transport::McpCallError;

    let (code, category, retryable) = match error {
        McpCallError::Unavailable => (
            "mcp_service_unavailable",
            ToolErrorCategory::Unavailable,
            true,
        ),
        McpCallError::Server => ("mcp_server_error", ToolErrorCategory::External, false),
        McpCallError::InvalidResponse => {
            ("mcp_invalid_response", ToolErrorCategory::External, false)
        }
        McpCallError::Transport => ("mcp_transport_failed", ToolErrorCategory::External, false),
    };
    let result = ToolResult::error(error.message(), code, category, retryable);
    if retryable {
        result.with_error_hint(
            "Aucun appel d'outil n'a été envoyé ; une nouvelle tentative est sûre.",
        )
    } else {
        result.with_error_hint(
            "Vérifier l'état du service avant de relancer : l'action a pu être exécutée.",
        )
    }
}

fn valid_tool_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn invalid_tool() -> ToolResult {
    ToolResult::error(
        "outil MCP invalide",
        "invalid_mcp_tool_id",
        ToolErrorCategory::Validation,
        false,
    )
}

fn to_tool_result(result: crate::services::mcp_bridge::transport::McpToolResult) -> ToolResult {
    let (content, truncated) = sanitize_output(&result.content);
    let mut output = if result.is_error {
        ToolResult::error(
            content,
            "mcp_tool_error",
            ToolErrorCategory::External,
            false,
        )
    } else {
        ToolResult::ok(content)
    };
    output.mark_truncated(truncated);
    output
}

fn sanitize_output(output: &str) -> (String, bool) {
    let mut sanitized = String::new();
    let mut truncated = false;
    for (count, character) in output
        .chars()
        .filter(|character| super::tool_result_contract::safe_metadata_character(*character))
        .enumerate()
    {
        if count == 4096 {
            truncated = true;
            break;
        }
        sanitized.push(character);
    }
    (sanitized, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::tool_result_contract::ToolResultStatus;
    use crate::services::mcp_bridge::transport::{McpCallError, McpToolResult};

    #[test]
    fn server_tool_errors_are_not_promoted_to_success() {
        let result = to_tool_result(McpToolResult {
            content: "invalid query".to_string(),
            is_error: true,
        });

        assert_eq!(result.status, ToolResultStatus::Error);
        assert_eq!(result.error.unwrap().code.as_ref(), "mcp_tool_error");
        assert_eq!(result.content, "invalid query");
    }

    #[test]
    fn oversized_output_reports_truncation() {
        let result = to_tool_result(McpToolResult {
            content: "a".repeat(4097),
            is_error: false,
        });

        assert_eq!(result.content.chars().count(), 4096);
        assert!(result.truncated);
    }

    #[test]
    fn bidi_controls_are_removed_without_marking_clean_output_truncated() {
        let result = to_tool_result(McpToolResult {
            content: "safe\u{061c}\u{200b}\u{202e}\u{2060}\u{feff}text".to_string(),
            is_error: false,
        });

        assert_eq!(result.content, "safetext");
        assert!(!result.truncated);
    }

    #[test]
    fn connector_and_protocol_failures_have_distinct_codes() {
        let server = transport_failure(McpCallError::Server);
        let protocol = transport_failure(McpCallError::InvalidResponse);
        let unavailable = transport_failure(McpCallError::Unavailable);

        let server_error = server.error.unwrap();
        let protocol_error = protocol.error.unwrap();
        assert_eq!(server_error.code.as_ref(), "mcp_server_error");
        assert_eq!(protocol_error.code.as_ref(), "mcp_invalid_response");
        assert!(!server_error.retryable);
        assert!(!protocol_error.retryable);
        assert!(unavailable.error.unwrap().retryable);
        assert!(protocol_error.hint.is_some());
    }
}
