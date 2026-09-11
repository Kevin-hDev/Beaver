use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct AutomationFile {
    pub schema_version: u32,
    automations: Vec<AutomationWire>,
}

impl AutomationFile {
    pub(super) fn from_definitions(automations: Vec<AutomationDefinition>) -> Self {
        Self {
            schema_version: super::store::AUTOMATIONS_SCHEMA_VERSION,
            automations: automations.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationWire {
    id: Uuid,
    revision: u64,
    name: String,
    description: Option<String>,
    prompt: String,
    creator_session_id: Option<String>,
    target_mode: TargetMode,
    target_session_id: Option<String>,
    project_id: Option<String>,
    provider: String,
    model: String,
    schedule: ScheduleWire,
    status: AutomationStatus,
    created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TargetMode {
    NewSession,
    ResumeSession,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ScheduleWire {
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

impl From<AutomationDefinition> for AutomationWire {
    fn from(value: AutomationDefinition) -> Self {
        let (target_mode, target_session_id, project_id) = match value.target {
            AutomationTarget::NewSession { project_id } => {
                (TargetMode::NewSession, None, project_id)
            }
            AutomationTarget::ResumeSession { session_id } => {
                (TargetMode::ResumeSession, Some(session_id), None)
            }
        };
        Self {
            id: value.id,
            revision: value.revision,
            name: value.name,
            description: value.description,
            prompt: value.prompt,
            creator_session_id: value.creator_session_id,
            target_mode,
            target_session_id,
            project_id,
            provider: value.provider,
            model: value.model,
            schedule: schedule_to_wire(value.schedule),
            status: value.status,
            created_at: value.created_at,
            anchor_at: value.anchor_at,
        }
    }
}

impl TryFrom<AutomationWire> for AutomationDefinition {
    type Error = String;

    fn try_from(value: AutomationWire) -> Result<Self, Self::Error> {
        let target = match (value.target_mode, value.target_session_id, value.project_id) {
            (TargetMode::NewSession, None, project_id) => {
                AutomationTarget::NewSession { project_id }
            }
            (TargetMode::ResumeSession, Some(session_id), None) if !session_id.is_empty() => {
                AutomationTarget::ResumeSession { session_id }
            }
            _ => return Err(super::store::store_error()),
        };
        Ok(Self {
            id: value.id,
            revision: value.revision,
            name: value.name,
            description: value.description,
            prompt: value.prompt,
            creator_session_id: value.creator_session_id,
            target,
            provider: value.provider,
            model: value.model,
            schedule: schedule_from_wire(value.schedule),
            status: value.status,
            created_at: value.created_at,
            anchor_at: value.anchor_at,
        })
    }
}

pub(super) fn decode_file(file: &AutomationFile) -> Result<Vec<AutomationDefinition>, String> {
    if file.schema_version != super::store::AUTOMATIONS_SCHEMA_VERSION {
        return Err(super::store::store_error());
    }
    file.automations
        .iter()
        .cloned()
        .map(TryInto::try_into)
        .collect()
}

fn schedule_to_wire(value: AutomationSchedule) -> ScheduleWire {
    match value {
        AutomationSchedule::Once {
            local_datetime,
            timezone,
        } => ScheduleWire::Once {
            local_datetime,
            timezone,
        },
        AutomationSchedule::Cron {
            expression,
            timezone,
        } => ScheduleWire::Cron {
            expression,
            timezone,
        },
        AutomationSchedule::AfterCompletion { delay_minutes } => {
            ScheduleWire::AfterCompletion { delay_minutes }
        }
    }
}

fn schedule_from_wire(value: ScheduleWire) -> AutomationSchedule {
    match value {
        ScheduleWire::Once {
            local_datetime,
            timezone,
        } => AutomationSchedule::Once {
            local_datetime,
            timezone,
        },
        ScheduleWire::Cron {
            expression,
            timezone,
        } => AutomationSchedule::Cron {
            expression,
            timezone,
        },
        ScheduleWire::AfterCompletion { delay_minutes } => {
            AutomationSchedule::AfterCompletion { delay_minutes }
        }
    }
}
