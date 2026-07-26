use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiagnosticRun {
    pub request_id: String,
    pub generation: u64,
    pub status: String,
    pub severity: String,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    pub phase: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_tool: Option<AgentDiagnosticTool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_tools: Vec<AgentDiagnosticTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo: Option<AgentDiagnosticTodo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<AgentDiagnosticEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiagnosticTool {
    pub name: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
    #[serde(default)]
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiagnosticTodo {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_task: Option<String>,
    pub completed: usize,
    pub total: usize,
    pub progress: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiagnosticEvent {
    pub at: DateTime<Utc>,
    pub phase: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentErrorDiagnosticSummary {
    pub request_id: String,
    pub phase: String,
    pub error_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_tool_name: Option<String>,
    pub safe_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStreamFailure {
    pub code: String,
    pub occurred_at: DateTime<Utc>,
    pub is_connection: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo_title: Option<String>,
}
