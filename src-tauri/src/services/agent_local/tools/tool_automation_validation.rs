#[path = "tool_automation_validation_wire.rs"]
mod wire;

use crate::models::{AutomationSchedule, AutomationStatus};
use crate::services::automations::{
    validate_multiline_text, validate_optional_multiline_text, validate_single_line_text,
    HistoryQuery, UpdateAutomation,
};
use chrono::NaiveDateTime;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug)]
pub(super) enum Action {
    List,
    Get(Uuid),
    Create(CreateRequest),
    Update(Uuid, UpdateAutomation),
    History(HistoryQuery),
    Delete(Uuid),
}

#[derive(Debug)]
pub(super) struct CreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub prompt: String,
    pub target_mode: TargetMode,
    pub model: Option<String>,
    pub schedule: AutomationSchedule,
    pub status: AutomationStatus,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum TargetMode {
    NewSession,
    ResumeSession,
}

pub(super) fn parse(args: &serde_json::Value) -> Result<Action, &'static str> {
    let request =
        serde_json::from_value::<wire::Request>(args.clone()).map_err(|_| "invalid_input")?;
    match request {
        wire::Request::List {} => Ok(Action::List),
        wire::Request::Get { automation_id } => Ok(Action::Get(automation_id)),
        wire::Request::Create {
            name,
            description,
            prompt,
            target_mode,
            model,
            schedule,
            status,
        } => {
            validate_single_line_text(&name, 120).map_err(|_| "invalid_input")?;
            validate_optional_multiline_text(description.as_deref(), 300)
                .map_err(|_| "invalid_input")?;
            validate_multiline_text(&prompt, 12_000).map_err(|_| "invalid_input")?;
            if let Some(model) = model.as_deref() {
                validate_single_line_text(model, 256).map_err(|_| "invalid_input")?;
            }
            Ok(Action::Create(CreateRequest {
                name: name.trim().to_string(),
                description,
                prompt: prompt.trim().to_string(),
                target_mode,
                model,
                schedule: schedule.try_into()?,
                status,
            }))
        }
        wire::Request::Update {
            automation_id,
            patch,
        } => Ok(Action::Update(automation_id, patch.try_into()?)),
        wire::Request::History {
            automation_id,
            limit,
            cursor,
        } => {
            validate_history(limit, cursor.as_deref())?;
            Ok(Action::History(HistoryQuery {
                automation_id,
                limit,
                cursor,
            }))
        }
        wire::Request::Delete { automation_id } => Ok(Action::Delete(automation_id)),
    }
}

impl TryFrom<wire::Update> for UpdateAutomation {
    type Error = &'static str;

    fn try_from(value: wire::Update) -> Result<Self, Self::Error> {
        let empty = value.name.is_none()
            && value.description.is_none()
            && value.prompt.is_none()
            && value.model.is_none()
            && value.schedule.is_none()
            && value.status.is_none();
        if empty {
            return Err("invalid_input");
        }
        if let Some(name) = value.name.as_deref() {
            validate_single_line_text(name, 120).map_err(|_| "invalid_input")?;
        }
        if let Some(prompt) = value.prompt.as_deref() {
            validate_multiline_text(prompt, 12_000).map_err(|_| "invalid_input")?;
        }
        if let Some(model) = value.model.as_deref() {
            validate_single_line_text(model, 256).map_err(|_| "invalid_input")?;
        }
        if let Some(description) = value.description.as_ref() {
            validate_optional_multiline_text(description.as_deref(), 300)
                .map_err(|_| "invalid_input")?;
        }
        Ok(UpdateAutomation {
            name: value.name,
            description: value.description,
            prompt: value.prompt,
            model: value.model,
            schedule: value.schedule.map(TryInto::try_into).transpose()?,
            status: value.status,
            ..Default::default()
        })
    }
}

impl TryFrom<wire::Schedule> for AutomationSchedule {
    type Error = &'static str;

    fn try_from(value: wire::Schedule) -> Result<Self, Self::Error> {
        match value {
            wire::Schedule::Once {
                local_datetime,
                timezone,
            } => Ok(Self::Once {
                local_datetime: parse_local_datetime(&local_datetime)?,
                timezone,
            }),
            wire::Schedule::Cron {
                expression,
                timezone,
            } if !expression.is_empty() && expression.chars().count() <= 256 => Ok(Self::Cron {
                expression,
                timezone,
            }),
            wire::Schedule::AfterCompletion { delay_minutes }
                if (1..=525_600).contains(&delay_minutes) =>
            {
                Ok(Self::AfterCompletion { delay_minutes })
            }
            _ => Err("invalid_schedule"),
        }
    }
}

fn parse_local_datetime(value: &str) -> Result<NaiveDateTime, &'static str> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
        .map_err(|_| "invalid_schedule")
}

fn validate_history(limit: Option<usize>, cursor: Option<&str>) -> Result<(), &'static str> {
    if limit.is_some_and(|limit| !(1..=100).contains(&limit))
        || cursor.is_some_and(|value| {
            value.is_empty() || value.chars().count() > 2_048 || value.chars().any(char::is_control)
        })
    {
        return Err("invalid_input");
    }
    Ok(())
}
