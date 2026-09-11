use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types_session::{CloneMode, SubagentLastActivity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionMeta {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<DateTime<Utc>>,
    /* Recopiée depuis la session : l'index n'est jamais l'autorité, il se
    reconstruit à partir des fichiers de session. */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_at: Option<DateTime<Utc>>,
    pub model: String,
    #[serde(default = "super::types_session::default_provider")]
    pub provider: String,
    #[serde(default)]
    pub thinking_enabled: bool,
    #[serde(default)]
    pub fast_mode_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_mode: Option<String>,
    pub message_count: usize,
    #[serde(default)]
    pub is_heartbeat: bool,
    #[serde(default)]
    pub is_gateway: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_channel_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_color_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_last_activity: Option<SubagentLastActivity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_parent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_parent_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_mode: Option<CloneMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_root_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
}
