use serde::{Deserialize, Serialize};

use super::types_message::AgentMessage;
use super::types_stream::TokenPhase;

pub(crate) const STREAM_RECOVERY_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StreamRecoveryHeader {
    pub version: u8,
    pub process_instance_id: String,
    pub session_id: String,
    pub request_id: String,
    pub turn_id: String,
    pub user_message_id: String,
    pub assistant_message_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_owner: Option<StreamRecoveryOwner>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StreamRecoveryOwner {
    pub run_id: String,
    pub execution_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum StreamRecoveryRecord {
    Header(StreamRecoveryHeader),
    Event {
        sequence: u64,
        event: RecoverableStreamEvent,
    },
    PendingMessage {
        sequence: u64,
        batch_id: String,
        position: usize,
        total: usize,
        message: AgentMessage,
    },
    TurnReady {
        sequence: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RecoverableStreamEvent {
    Token { content: String, phase: Option<TokenPhase> },
    Thinking { content: String },
    ContentPhase { phase: TokenPhase },
    AttemptRestarted { reason_key: String, attempt: u32 },
    ToolCall(RecoverableToolCall),
    ToolResult(RecoverableToolResult),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoverableToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
    pub tool_call_index: usize,
    pub tool_call_id: Option<String>,
    pub domain: Option<String>,
    pub extra_content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoverableToolResult {
    pub name: String,
    pub content: String,
    pub is_error: bool,
    pub status: super::tool_result_contract::ToolResultStatus,
    pub error: Option<super::tool_result_contract::ToolErrorInfo>,
    pub warnings: Vec<String>,
    pub truncated: bool,
    pub display_summary: Option<String>,
    pub tool_call_index: usize,
    pub tool_call_id: Option<String>,
    pub resolved_path: Option<String>,
    pub domain: Option<String>,
    pub affected_paths: Vec<String>,
    pub file_changes: Vec<super::types_tools::ToolFileChange>,
    pub start_line: Option<usize>,
    pub artifacts: Vec<super::tool_artifact_record::ToolArtifactRecord>,
    pub follow_up: RecoverableToolFollowUp,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "kind", content = "content", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RecoverableToolFollowUp {
    #[default]
    None,
    UserMessage(String),
    SystemMessage(String),
    Stop,
}

pub(crate) fn process_instance_id() -> &'static str {
    static ID: std::sync::LazyLock<String> =
        std::sync::LazyLock::new(|| uuid::Uuid::new_v4().to_string());
    ID.as_str()
}

pub(crate) fn validate_header(header: &StreamRecoveryHeader) -> Result<(), String> {
    if header.version != STREAM_RECOVERY_VERSION {
        return Err(error());
    }
    for id in [
        &header.process_instance_id,
        &header.session_id,
        &header.request_id,
        &header.turn_id,
        &header.user_message_id,
        &header.assistant_message_id,
    ] {
        uuid::Uuid::parse_str(id).map_err(|_| error())?;
    }
    if let Some(owner) = &header.subagent_owner {
        uuid::Uuid::parse_str(&owner.run_id).map_err(|_| error())?;
        uuid::Uuid::parse_str(&owner.execution_id).map_err(|_| error())?;
    }
    Ok(())
}

fn error() -> String {
    "stream_recovery_invalid".to_string()
}
