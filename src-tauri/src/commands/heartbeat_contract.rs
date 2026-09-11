use crate::models::AutomationStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScheduleInput {
    Once {
        local_datetime: String,
        timezone: String,
    },
    Cron {
        expression: String,
        timezone: String,
    },
    AfterCompletion {
        delay_minutes: u32,
    },
}

#[derive(Debug, Deserialize)]
pub struct CreateWakeupInput {
    pub name: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub schedule: ScheduleInput,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWakeupInput {
    pub automation_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "present_optional")]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub schedule: Option<ScheduleInput>,
    #[serde(default)]
    pub status: Option<AutomationStatus>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictDecision {
    RemoveHistorical,
    ImportAsNew,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct MigrationStatusView {
    pub status: &'static str,
    pub conflicts: Vec<crate::services::automations::migration::MigrationConflict>,
}

fn present_optional<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}
