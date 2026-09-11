use crate::models::AutomationSchedule;
use chrono::NaiveDateTime;
use chrono_tz::Tz;
use std::str::FromStr;

pub(super) fn schedule(
    input: super::heartbeat::ScheduleInput,
) -> Result<AutomationSchedule, String> {
    let schedule = match input {
        super::heartbeat::ScheduleInput::Once {
            local_datetime,
            timezone: timezone_name,
        } => AutomationSchedule::Once {
            local_datetime: parse_datetime(&local_datetime)?,
            timezone: timezone(&timezone_name)?,
        },
        super::heartbeat::ScheduleInput::Cron {
            expression,
            timezone: timezone_name,
        } => AutomationSchedule::Cron {
            expression,
            timezone: timezone(&timezone_name)?,
        },
        super::heartbeat::ScheduleInput::AfterCompletion { delay_minutes } => {
            AutomationSchedule::AfterCompletion { delay_minutes }
        }
    };
    crate::services::automations::validate_schedule(&schedule)
        .map_err(|error| error.code().to_string())?;
    Ok(schedule)
}

pub(super) fn timezone(value: &str) -> Result<Tz, String> {
    Tz::from_str(value).map_err(|_| "invalid_timezone".into())
}

fn parse_datetime(value: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
        .map_err(|_| "invalid_schedule".into())
}
