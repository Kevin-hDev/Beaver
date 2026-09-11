use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationOrigin {
    Session,
    ExternalChannel,
    UserInterface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationActor {
    pub origin: AutomationOrigin,
    pub session_or_channel_id: String,
    pub current_automation_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct CreateAutomation {
    pub name: String,
    pub description: Option<String>,
    pub prompt: String,
    pub target: AutomationTarget,
    pub provider: String,
    pub model: String,
    pub schedule: AutomationSchedule,
    pub status: AutomationStatus,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateAutomation {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub schedule: Option<AutomationSchedule>,
    pub status: Option<AutomationStatus>,
    pub target: Option<AutomationTarget>,
    pub provider: Option<String>,
    pub creator_session_id: Option<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationSummary {
    pub id: Uuid,
    pub revision: u64,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub target: AutomationTarget,
    pub schedule: AutomationSchedule,
    pub status: AutomationStatus,
    pub running: bool,
    pub paused_by_global: bool,
    pub next_fire_at: Option<DateTime<Utc>>,
    pub last_run: Option<AutomationLastRun>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationLastRun {
    pub status: crate::models::WakeupRunStatus,
    pub finished_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationDetail {
    pub definition: AutomationDefinition,
    pub next_fire_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct HistoryQuery {
    pub automation_id: Uuid,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<Uuid>,
    #[serde(alias = "wakeup_id")]
    pub automation_id: String,
    pub scheduled_for: String,
    #[serde(alias = "fired_at")]
    pub finished_at: String,
    pub status: crate::models::WakeupRunStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<crate::models::WakeupRunErrorCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missed_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_scheduled_for: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_scheduled_for: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryPage {
    pub entries: Vec<HistoryEntry>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationError {
    NotFound,
    CapacityReached,
    InvalidInput,
    ImmutableField,
    InvalidSchedule,
    ProviderUnavailable,
    ModelUnavailable,
    ModelToolsUnsupported,
    AuditUnavailable,
    StoreUnavailable,
    CursorExpired,
}

impl std::fmt::Display for AutomationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.code())
    }
}

impl std::error::Error for AutomationError {}

impl AutomationError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::CapacityReached => "capacity_reached",
            Self::InvalidInput => "invalid_input",
            Self::ImmutableField => "immutable_field",
            Self::InvalidSchedule => "invalid_schedule",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::ModelUnavailable => "model_unavailable",
            Self::ModelToolsUnsupported => "model_tools_unsupported",
            Self::AuditUnavailable => "audit_unavailable",
            Self::StoreUnavailable => "store_unavailable",
            Self::CursorExpired => "cursor_expired",
        }
    }
}
