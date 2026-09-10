use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationDefinition {
    pub id: Uuid,
    pub revision: u64,
    pub name: String,
    pub description: Option<String>,
    pub prompt: String,
    pub creator_session_id: Option<String>,
    pub target: AutomationTarget,
    pub provider: String,
    pub model: String,
    pub schedule: AutomationSchedule,
    pub status: AutomationStatus,
    pub created_at: DateTime<Utc>,
    pub anchor_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationTarget {
    NewSession { project_id: Option<String> },
    ResumeSession { session_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationSchedule {
    Once {
        local_datetime: NaiveDateTime,
        timezone: Tz,
    },
    Cron {
        expression: String,
        timezone: Tz,
    },
    AfterCompletion {
        delay_minutes: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationStatus {
    Active,
    Disabled,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledWakeup {
    pub id: String,
    pub name: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub schedule: WakeupSchedule,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub project_id: Option<String>,
    pub active: bool,
    #[serde(default)]
    pub paused_by_global: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum WakeupSchedule {
    Once { datetime: String },
    Daily { time: String },
    Weekly { weekday: u8, time: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WakeupRunStatus {
    Ok,
    Error,
    Missed,
    Cancelled,
    Interrupted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WakeupRunErrorCode {
    Failed,
    RateLimited,
    AuthenticationFailed,
    OllamaUnavailable,
    MissedUnavailable,
    SchedulerStopping,
    CapacityReached,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupRun {
    pub wakeup_id: String,
    pub scheduled_for: String,
    pub fired_at: String,
    pub status: WakeupRunStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<WakeupRunErrorCode>,
    #[serde(rename = "error", default, skip_serializing)]
    pub _legacy_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupStatusSummary {
    pub wakeup_id: String,
    pub next_fire_at: Option<String>,
    pub last_run: Option<WakeupRun>,
}
