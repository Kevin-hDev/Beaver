use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types_stream::TokenPhase;
use super::{
    tool_result_contract::{ToolErrorInfo, ToolResultStatus},
    types_tools::ToolFileChange,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
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
        validate_file_changes(self)
    }
}

fn validate_file_changes(message: &AgentMessage) -> Result<(), String> {
    let records = message.tool_activities.iter().flatten().chain(
        message
            .segments
            .iter()
            .flatten()
            .flat_map(|segment| &segment.tools),
    );
    for record in records {
        if record.domain.as_deref().is_some_and(|domain| domain != "memory")
            || record.resolved_path.as_deref().is_some_and(|path| {
                path.is_empty() || path.len() > 4_096 || path.contains('\0')
            })
        {
            return Err("Historique d'outil invalide.".to_string());
        }
        if record.file_changes.len() > super::tool_file_changes::MAX_FILE_CHANGES {
            return Err("Historique de fichiers invalide.".to_string());
        }
        if record.affected_paths.len() > super::types_tool_result_details::MAX_AFFECTED_PATHS
            || record.affected_paths.iter().any(|path| {
                path.is_empty() || path.len() > 4_096 || path.contains('\0')
            })
        {
            return Err("Historique de fichiers invalide.".to_string());
        }
        if record.result_meta.as_ref().is_some_and(|meta| {
            meta.warnings.len() > 16
                || meta.status.is_error() != record.is_error.unwrap_or(false)
                || meta.status.is_error() != meta.error.is_some()
                || meta.warnings.iter().any(|warning| !valid_meta_text(warning))
                || meta.error.as_ref().is_some_and(|error| {
                    error.code.is_empty()
                        || error.code.len() > 100
                        || !error.code.bytes().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                        })
                        || error.hint.as_ref().is_some_and(|hint| !valid_meta_text(hint))
                })
        }) {
            return Err("Historique de résultat d'outil invalide.".to_string());
        }
        let mut total_diff_bytes = 0usize;
        for change in &record.file_changes {
            if let Some(diff) = &change.diff {
                total_diff_bytes = total_diff_bytes.saturating_add(
                    crate::services::git::diff_preview::preview_content_bytes(diff),
                );
            }
            if change.path.is_empty()
                || change.path.len() > 4_096
                || change.path.contains('\0')
                || change.additions > 2_000
                || change.deletions > 2_000
                || total_diff_bytes > super::tool_file_changes::MAX_FILE_CHANGE_DIFF_BYTES
                || change.diff.as_ref().is_some_and(|diff| {
                    !crate::services::git::diff_preview::is_bounded_preview(diff)
                })
            {
                return Err("Historique de fichiers invalide.".to_string());
            }
        }
    }
    Ok(())
}

fn valid_meta_text(text: &str) -> bool {
    super::tool_result_contract::is_safe_metadata_text(text, 1_000)
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
