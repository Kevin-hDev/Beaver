use super::types::{ExtensionContributions, ExtensionManifest};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostExtensionSpec {
    pub id: String,
    pub main_path: String,
    pub manifest: ExtensionManifest,
}

#[derive(Debug, Deserialize)]
pub struct HelloResult {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    #[serde(rename = "jitiVersion")]
    pub jiti_version: String,
    #[serde(rename = "nodeVersion")]
    pub node_version: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncResult {
    pub extensions: Vec<LoadedExtension>,
}

#[derive(Debug, Deserialize)]
pub struct LoadedExtension {
    pub id: String,
    pub contributions: Option<ExtensionContributions>,
    pub error: Option<String>,
    pub diagnostic: Option<HostDiagnostic>,
}

#[derive(Debug, Deserialize)]
pub struct HostDiagnostic {
    pub stage: String,
    pub code: String,
    pub file: Option<String>,
    pub line: Option<u64>,
    pub column: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostToolResult {
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub truncated: bool,
    pub display_summary: Option<String>,
}

#[derive(Serialize)]
pub struct RpcRequest<'a> {
    pub jsonrpc: &'static str,
    pub id: &'a str,
    pub method: &'a str,
    pub params: Value,
}

#[derive(Serialize)]
pub struct RpcResult<'a, T: Serialize> {
    pub jsonrpc: &'static str,
    pub id: &'a str,
    pub result: T,
}

#[derive(Serialize)]
pub struct RpcError<'a> {
    pub jsonrpc: &'static str,
    pub id: &'a str,
    pub error: RpcErrorBody,
}

#[derive(Serialize)]
pub struct RpcErrorBody {
    pub code: i32,
    pub message: &'static str,
}

pub fn envelope(value: &Value) -> Result<&Map<String, Value>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || object.get("id").and_then(Value::as_str).is_none()
    {
        return Err("Réponse de l'hôte d'extensions invalide.".to_string());
    }
    Ok(object)
}
