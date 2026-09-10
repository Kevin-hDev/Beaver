use crate::models::{
    AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget, ScheduledWakeup,
    WakeupSchedule,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use serde_json::Value;
use uuid::Uuid;

pub(super) fn convert(raw: &Value, timezone: Tz) -> Result<AutomationDefinition, String> {
    let legacy: ScheduledWakeup =
        serde_json::from_value(raw.clone()).map_err(|_| migration_error())?;
    let schedule = convert_schedule(&legacy.schedule, timezone)?;
    let created_at = DateTime::parse_from_rfc3339(&legacy.created_at)
        .map_err(|_| migration_error())?
        .with_timezone(&Utc);
    Ok(AutomationDefinition {
        id: Uuid::parse_str(&legacy.id).map_err(|_| migration_error())?,
        revision: 1,
        name: legacy.name,
        description: (!legacy.description.is_empty()).then_some(legacy.description),
        prompt: legacy.prompt,
        creator_session_id: None,
        target: AutomationTarget::NewSession {
            project_id: legacy.project_id,
        },
        provider: legacy.provider,
        model: legacy.model,
        schedule,
        status: if legacy.active {
            AutomationStatus::Active
        } else {
            AutomationStatus::Disabled
        },
        created_at,
        anchor_at: None,
    })
}

fn convert_schedule(value: &WakeupSchedule, timezone: Tz) -> Result<AutomationSchedule, String> {
    match value {
        WakeupSchedule::Once { datetime } => Ok(AutomationSchedule::Once {
            local_datetime: NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M")
                .map_err(|_| migration_error())?,
            timezone,
        }),
        WakeupSchedule::Daily { time } => Ok(AutomationSchedule::Cron {
            expression: format!("{} {} * * *", minute(time)?, hour(time)?),
            timezone,
        }),
        WakeupSchedule::Weekly { weekday, time } if *weekday <= 6 => Ok(AutomationSchedule::Cron {
            expression: format!("{} {} * * {}", minute(time)?, hour(time)?, weekday),
            timezone,
        }),
        WakeupSchedule::Weekly { .. } => Err(migration_error()),
    }
}

fn hour(value: &str) -> Result<u8, String> {
    parse_time(value).map(|(hour, _)| hour)
}

fn minute(value: &str) -> Result<u8, String> {
    parse_time(value).map(|(_, minute)| minute)
}

fn parse_time(value: &str) -> Result<(u8, u8), String> {
    let (hour, minute) = value.split_once(':').ok_or_else(migration_error)?;
    let hour = hour.parse::<u8>().map_err(|_| migration_error())?;
    let minute = minute.parse::<u8>().map_err(|_| migration_error())?;
    if hour > 23 || minute > 59 {
        return Err(migration_error());
    }
    Ok((hour, minute))
}

fn migration_error() -> String {
    "AUTOMATION_MIGRATION_UNAVAILABLE".into()
}
