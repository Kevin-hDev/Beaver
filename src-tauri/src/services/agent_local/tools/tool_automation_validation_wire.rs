use crate::models::AutomationStatus;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Request {
    List {},
    Get {
        automation_id: Uuid,
    },
    Create {
        name: String,
        #[serde(default)]
        description: Option<String>,
        prompt: String,
        target_mode: super::TargetMode,
        #[serde(default)]
        model: Option<String>,
        schedule: Schedule,
        #[serde(default = "active")]
        status: AutomationStatus,
    },
    Update {
        automation_id: Uuid,
        patch: Update,
    },
    History {
        automation_id: Uuid,
        #[serde(default)]
        limit: Option<usize>,
        #[serde(default)]
        cursor: Option<String>,
    },
    Delete {
        automation_id: Uuid,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Update {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "present_optional")]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub schedule: Option<Schedule>,
    #[serde(default)]
    pub status: Option<AutomationStatus>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Schedule {
    Once {
        local_datetime: String,
        timezone: chrono_tz::Tz,
    },
    Cron {
        expression: String,
        timezone: chrono_tz::Tz,
    },
    AfterCompletion {
        delay_minutes: u32,
    },
}

fn active() -> AutomationStatus {
    AutomationStatus::Active
}

fn present_optional<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}
