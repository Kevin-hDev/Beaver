use serde_json::{json, Value};

use super::BuildError;
use crate::services::llm::route_profile::SchemaPolicy;

pub(super) fn convert(tools: &[Value]) -> Result<Vec<Value>, BuildError> {
    if tools.len() > crate::services::agent_local::provider_tool_limits::COMPATIBILITY_TOOL_LIMIT {
        return Err(BuildError::TooManyTools);
    }
    let normalized =
        crate::services::llm::tool_schema::tools_for_policy(SchemaPolicy::Anthropic, false, tools);
    normalized
        .iter()
        .map(|tool| {
            let function = tool
                .get("function")
                .and_then(Value::as_object)
                .ok_or(BuildError::InvalidToolSchema)?;
            let name = function
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or(BuildError::InvalidToolSchema)?;
            let schema = function
                .get("parameters")
                .and_then(Value::as_object)
                .ok_or(BuildError::InvalidToolSchema)?;
            let description = function
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            Ok(json!({
                "name": name,
                "description": description,
                "input_schema": schema,
            }))
        })
        .collect()
}
