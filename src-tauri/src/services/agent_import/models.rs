use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source_id: String,
    pub source_name: String,
    pub kind: ImportItemKind,
    pub selected: bool,
    pub available: bool,
    pub update_available: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportItemKind {
    Document,
    Rule,
    Skill,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSourceSummary {
    pub id: String,
    pub display_name: String,
    pub status: SourceStatus,
    pub partial: bool,
    pub configured: bool,
    pub enabled: bool,
    pub documents: Vec<ImportItem>,
    pub rules: Vec<ImportItem>,
    pub skills: Vec<ImportItem>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceStatus {
    Detected,
    Empty,
    Missing,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SelectionMode {
    All,
    None,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSelection {
    pub source_id: String,
    pub enabled: bool,
    pub skill_mode: SelectionMode,
    #[serde(default)]
    pub selected_skill_ids: Vec<String>,
    #[serde(default)]
    pub selected_rule_ids: Vec<String>,
    #[serde(default)]
    pub selected_document_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSelectionResult {
    pub saved: bool,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredItem {
    pub public: ImportItem,
    pub path: PathBuf,
    pub bundle_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredSource {
    pub summary: AgentSourceSummary,
    pub documents: Vec<DiscoveredItem>,
    pub rules: Vec<DiscoveredItem>,
    pub skills: Vec<DiscoveredItem>,
}
