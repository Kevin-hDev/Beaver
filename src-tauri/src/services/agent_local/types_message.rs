use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types_stream::TokenPhase;
use super::{
    tool_result_contract::{ToolErrorInfo, ToolResultStatus},
    types_tools::ToolFileChange,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum AgentMessageKind {
    CompressionCheckpoint,
    CompressionBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    #[serde(default = "AgentMessage::new_turn_id")]
    pub turn_id: String,
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_kind: Option<AgentMessageKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::types_message_continuation::deserialize"
    )]
    pub continuation: Option<crate::services::reasoning_continuity::envelope::ReasoningEnvelope>,
    /// Provenance privée du tour, durable même si aucune enveloppe n'est capturée.
    /// Elle ne fait volontairement pas partie des contrats IPC visibles.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::types_message_source::deserialize"
    )]
    pub replay_source: Option<crate::services::reasoning_continuity::envelope::ReasoningSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_activities: Option<Vec<ToolActivityRecord>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<SavedSegment>>,
    pub files: Vec<FileAttachment>,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_names: Option<Vec<String>>,
    /// Identifiants privés nécessaires pour recharger les skills côté Rust.
    /// Ils ne font volontairement pas partie des contrats IPC visibles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_part: Option<String>,
}

impl AgentMessage {
    pub fn validate_stream_metadata(&self) -> Result<(), String> {
        let metadata = match (&self.stream_run_id, &self.stream_part) {
            (None, None) => Ok(()),
            (Some(run_id), Some(part)) => {
                uuid::Uuid::parse_str(run_id)
                    .map_err(|_| "Metadonnees de message invalides.".to_string())?;
                if matches!(part.as_str(), "checkpoint" | "input" | "final") {
                    Ok(())
                } else {
                    Err("Metadonnees de message invalides.".to_string())
                }
            }
            _ => Err("Metadonnees de message invalides.".to_string()),
        };
        metadata?;
        super::types_message_validation::validate(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolActivityRecord {
    pub name: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_meta: Option<PersistedToolResultMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_changes: Vec<ToolFileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedToolResultMeta {
    pub status: ToolResultStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolErrorInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSegment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    pub tools: Vec<ToolActivityRecord>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<TokenPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    #[serde(default = "ToolCallRequest::local_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub extra_content: Option<serde_json::Value>,
    pub function: ToolCallRequestFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequestFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub name: String,
    pub path: String,
    pub mime_type: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_grant: Option<String>,
}

#[cfg(test)]
#[path = "types_message_tests.rs"]
mod tests;
