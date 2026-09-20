use serde_json::Value;

use super::transport::{McpCallError, McpToolResult};

pub fn complete(value: &Value) -> Result<McpToolResult, McpCallError> {
    super::schema_limits::validate(value).map_err(|_| McpCallError::InvalidResponse)?;
    let object = value.as_object().ok_or(McpCallError::InvalidResponse)?;
    match object.get("resultType") {
        None => {}
        Some(Value::String(kind)) if kind == "complete" => {}
        _ => return Err(McpCallError::InvalidResponse),
    }
    let is_error = match object.get("isError") {
        None => false,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err(McpCallError::InvalidResponse),
    };

    if object.get("structuredContent").is_none() {
        if let Some(content) = object.get("content").and_then(Value::as_array) {
            let texts: Option<Vec<&str>> = content
                .iter()
                .map(|item| {
                    if item.get("type").is_some_and(|kind| kind != "text") {
                        None
                    } else {
                        item.get("text").and_then(Value::as_str)
                    }
                })
                .collect();
            if let Some(texts) = texts.filter(|texts| !texts.is_empty()) {
                return Ok(McpToolResult {
                    content: texts.join("\n"),
                    is_error,
                });
            }
        }
    }

    let content = serde_json::to_string_pretty(value).map_err(|_| McpCallError::InvalidResponse)?;
    Ok(McpToolResult { content, is_error })
}
