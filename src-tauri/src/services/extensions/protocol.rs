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
#[serde(deny_unknown_fields)]
pub struct LoadResult {
    pub id: String,
    pub contributions: Option<ExtensionContributions>,
    pub error: Option<String>,
    pub diagnostic: Option<HostDiagnostic>,
    #[serde(default, rename = "uiDiagnostics")]
    pub ui_diagnostics: Vec<HostUiDiagnostic>,
}

pub struct AttributedLoadResult {
    pub identity: super::host_identity::HostIdentity,
    pub generation: u64,
    pub loaded: LoadResult,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostUiDiagnostic {
    pub code: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
    let identifier = object.get("id");
    let method = object.get("method");
    let valid_identifier = identifier.is_some_and(|value| {
        value
            .as_str()
            .is_some_and(super::host_channel::valid_request_id)
    });
    let valid_method = method.is_some_and(|value| value.as_str().is_some());
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || identifier.is_some_and(|_| !valid_identifier)
        || method.is_some_and(|_| !valid_method)
        || (!valid_identifier && !valid_method)
    {
        return Err("Réponse de l'hôte d'extensions invalide.".to_string());
    }
    Ok(object)
}
