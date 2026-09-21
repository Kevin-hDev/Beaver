use serde_json::{json, Value};

use super::core_bridge::{string_param, CoreResponse, ExtensionBridgeError};
use crate::services::mcp_bridge::transport::McpCallError;

pub(super) async fn call(params: &Value) -> Result<CoreResponse, ExtensionBridgeError> {
    let connector_id =
        string_param(params, "connectorId").map_err(|_| ExtensionBridgeError::Failed)?;
    let tool_name = string_param(params, "toolName").map_err(|_| ExtensionBridgeError::Failed)?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    super::validation::message(&arguments).map_err(|_| ExtensionBridgeError::Failed)?;
    let (connector, tool) =
        crate::services::mcp_bridge::registry::resolve_enabled_tool(connector_id, tool_name)
            .await
            .map_err(|_| ExtensionBridgeError::Backend("core_mcp_unavailable"))?;
    crate::services::mcp_bridge::arguments::validate(&arguments, tool.input_schema.as_ref())
        .map_err(|_| ExtensionBridgeError::Failed)?;
    let result =
        crate::services::mcp_bridge::registry::call_enabled_tool(&connector, &tool.name, arguments)
            .await
            .map_err(|error| match error {
                McpCallError::Unavailable | McpCallError::ReauthenticationRequired => {
                    ExtensionBridgeError::Backend("core_mcp_unavailable")
                }
                McpCallError::Server | McpCallError::InvalidResponse | McpCallError::Transport => {
                    ExtensionBridgeError::Backend("core_mcp_result_unconfirmed")
                }
            })?;
    if result.is_error {
        return Err(ExtensionBridgeError::Backend("core_mcp_tool_error"));
    }
    Ok(CoreResponse::Json(Value::String(result.content)))
}
