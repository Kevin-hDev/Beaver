use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

pub static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

pub const MAX_TOOLS: usize = 128;
pub const MAX_DESC_CHARS: usize = 250;
pub const MAX_NAME_CHARS: usize = 64;

#[derive(Clone, Deserialize)]
pub struct McpToolDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

fn validate_tool_def(tool: &mut McpToolDef) -> Result<(), String> {
    if tool.name.is_empty()
        || tool.name.len() > MAX_NAME_CHARS
        || !tool
            .name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err("catalogue MCP invalide".to_string());
    }
    if let Some(ref desc) = tool.description {
        let sanitized: String = desc
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .take(MAX_DESC_CHARS)
            .collect();
        tool.description = Some(sanitized);
    }
    let schema = tool
        .input_schema
        .as_ref()
        .ok_or_else(|| "catalogue MCP invalide".to_string())?;
    super::schema::validate_definition(schema).map_err(|_| "catalogue MCP invalide".to_string())
}

pub fn validate_tools(mut tools: Vec<McpToolDef>) -> Result<Vec<McpToolDef>, String> {
    if tools.len() > MAX_TOOLS {
        return Err("catalogue MCP invalide".to_string());
    }
    let mut names = HashSet::with_capacity(tools.len());
    for tool in &mut tools {
        validate_tool_def(tool)?;
        if !names.insert(tool.name.clone()) {
            return Err("catalogue MCP invalide".to_string());
        }
    }
    Ok(tools)
}

pub fn extract_tool_result(resp: &Value) -> Result<String, String> {
    if resp.get("error").is_some() {
        return Err("erreur MCP retournée par le connecteur".to_string());
    }

    let result = resp.get("result").ok_or("réponse vide du serveur MCP")?;
    super::schema_limits::validate(result).map_err(|_| "réponse MCP invalide".to_string())?;

    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
        let texts: Vec<&str> = content
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .collect();
        if !texts.is_empty() {
            return Ok(texts.join("\n"));
        }
    }

    Ok(serde_json::to_string_pretty(result).unwrap_or_default())
}

#[async_trait]
pub trait McpTransport: Send + Sync {
    async fn list_tools(&self) -> Result<Vec<McpToolDef>, String>;
    async fn call_tool(&self, name: &str, args: Value) -> Result<String, String>;
}
