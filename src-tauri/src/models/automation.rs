use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum AutomationTarget {
    NewSession { project_id: Option<String> },
    ResumeSession { session_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
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
    AppStopped,
    TargetSessionMissing,
    ProviderUnavailable,
    ModelUnavailable,
}
