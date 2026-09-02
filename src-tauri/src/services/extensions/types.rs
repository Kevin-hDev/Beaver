use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MINIMUM_NODE_MAJOR: u64 = 20;
include!(concat!(env!("OUT_DIR"), "/extension_contract.rs"));

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionKind {
    Builtin,
    Local,
    External,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub beaver_api: String,
    pub runtime: String,
    pub main: Option<String>,
    pub ui: Option<String>,
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
    pub show_in_chat: bool,
    pub status: ExtensionStatus,
    pub last_error: Option<String>,
    pub last_activated_at: Option<String>,
    #[serde(skip)]
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
    pub file: Option<String>,
    pub line: Option<u64>,
    pub column: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HostState {
    Stopped,
    Starting,
    Running,
    Error,
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
