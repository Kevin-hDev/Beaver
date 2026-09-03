use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MINIMUM_NODE_MAJOR: u64 = 20;
include!(concat!(env!("OUT_DIR"), "/extension_contract.rs"));

/// Un chargement peut produire un diagnostic de manifeste, d'Hôte, d'UI et
/// de limite par extension. La projection d'état ne grandit jamais au-delà.
pub const MAX_RUNTIME_DIAGNOSTICS: usize = MAX_EXTENSIONS * 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionKind {
    Builtin,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionOriginKind {
    Local,
    Git,
    Npm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionOrigin {
    pub kind: ExtensionOriginKind,
    pub locator: String,
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionStatus {
    Active,
    Inactive,
    Loading,
    Error,
    Incompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionApiLevel {
    Stable,
    Advanced,
}

fn default_api_level() -> ExtensionApiLevel {
    ExtensionApiLevel::Stable
}

fn default_access() -> String {
    "full".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionUiMode {
    Standard,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionUiManifest {
    pub api_version: String,
    pub mode: ExtensionUiMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionUiArtifactOutput {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub bytes: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionUiArtifact {
    pub version: u8,
    pub builder_version: String,
    pub node_version: String,
    pub entry: String,
    pub total_bytes: usize,
    pub outputs: Vec<ExtensionUiArtifactOutput>,
    pub inputs: Vec<String>,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub beaver_api: String,
    pub runtime: String,
    pub main: Option<String>,
    pub ui: Option<ExtensionUiManifest>,
    /// Projection transitoire de la chaîne UI v1 : lisible pour le diagnostic,
    /// mais distincte du manifeste structuré afin de ne jamais réactiver l'UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_legacy: Option<String>,
    #[serde(default = "default_access")]
    pub access: String,
    #[serde(default = "default_api_level")]
    pub api_level: ExtensionApiLevel,
    #[serde(default)]
    pub essential: bool,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionContributions {
    #[serde(default)]
    pub tools: Vec<ExtensionTool>,
    #[serde(default)]
    pub events: Vec<String>,
    /// Transport Hôte -> cœur uniquement. Le catalogue UI possède sa propre
    /// autorité mémoire et cette valeur n'est jamais sérialisée vers l'UI.
    #[serde(default, skip_serializing)]
    pub ui: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionTool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    #[serde(default)]
    pub effect: ExtensionEffect,
    #[serde(default)]
    pub replaces_core: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionRecord {
    pub manifest: ExtensionManifest,
    pub kind: ExtensionKind,
    pub source: String,
    #[serde(default)]
    pub origin: Option<ExtensionOrigin>,
    pub enabled: bool,
    #[serde(default)]
    pub trusted: bool,
    #[serde(default)]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_artifact: Option<ExtensionUiArtifact>,
    #[serde(default)]
    pub trusted_at: Option<String>,
    pub show_in_chat: bool,
    pub status: ExtensionStatus,
    pub last_error: Option<String>,
    pub last_activated_at: Option<String>,
    #[serde(default)]
    pub sensitive_access_granted: bool,
    #[serde(skip)]
    pub contributions: ExtensionContributions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDiagnostic {
    pub extension_id: String,
    pub stage: String,
    pub code: String,
    pub occurred_at: String,
    pub file: Option<String>,
    pub line: Option<u64>,
    pub column: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionHostStatus {
    pub state: HostState,
    pub node_version: Option<String>,
    pub jiti_version: String,
    pub api_version: String,
    pub active_extensions: usize,
    pub last_error: Option<String>,
    pub diagnostics: Vec<ExtensionDiagnostic>,
}

impl Default for ExtensionHostStatus {
    fn default() -> Self {
        Self {
            state: HostState::Stopped,
            node_version: None,
            jiti_version: String::new(),
            api_version: BEAVER_API_VERSION.to_string(),
            active_extensions: 0,
            last_error: None,
            diagnostics: Vec::new(),
        }
    }
}
