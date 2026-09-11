use super::{CreateWakeupInput, UpdateWakeupInput};
use crate::commands::heartbeat_validation as validation;
use crate::models::{AutomationStatus, AutomationTarget};
use crate::services::automations::{
    AutomationActor, AutomationDetail, AutomationError, AutomationOrigin, AutomationSummary,
    CreateAutomation, HistoryPage, HistoryQuery, UpdateAutomation,
};
use uuid::Uuid;

pub(super) async fn list() -> Result<Vec<AutomationSummary>, String> {
    crate::services::automations::list(&ui_actor())
        .await
        .map_err(command_error)
}

pub(super) async fn get(id: Uuid) -> Result<AutomationDetail, String> {
    crate::services::automations::get(&ui_actor(), id)
        .await
        .map_err(command_error)
}

pub(super) async fn create(input: CreateWakeupInput) -> Result<AutomationDetail, String> {
    validate_project(input.project_id.as_deref()).await?;
    crate::services::automations::create(
        &ui_actor(),
        CreateAutomation {
            name: input.name,
            description: input.description,
            prompt: input.prompt,
            target: AutomationTarget::NewSession {
                project_id: input.project_id,
            },
            provider: input.provider,
            model: input.model,
            schedule: validation::schedule(input.schedule)?,
            status: AutomationStatus::Active,
        },
    )
    .await
    .map_err(command_error)
}

pub(super) async fn update(input: UpdateWakeupInput) -> Result<AutomationDetail, String> {
    let schedule = input.schedule.map(validation::schedule).transpose()?;
    crate::services::automations::update(
        &ui_actor(),
        input.automation_id,
        UpdateAutomation {
            name: input.name,
            description: input.description,
            prompt: input.prompt,
            model: input.model,
            schedule,
            status: input.status,
            ..Default::default()
        },
    )
    .await
    .map_err(command_error)
}

pub(super) async fn set_active(id: Uuid, active: bool) -> Result<AutomationDetail, String> {
    crate::services::automations::update(
        &ui_actor(),
        id,
        UpdateAutomation {
            status: Some(if active {
                AutomationStatus::Active
            } else {
                AutomationStatus::Disabled
            }),
            ..Default::default()
        },
    )
    .await
    .map_err(command_error)
}

pub(super) async fn delete(id: Uuid) -> Result<(), String> {
    crate::services::automations::delete(&ui_actor(), id)
        .await
        .map_err(command_error)
}

pub(super) async fn history(
    id: Uuid,
    limit: Option<usize>,
    cursor: Option<String>,
) -> Result<HistoryPage, String> {
    crate::services::automations::history(
        &ui_actor(),
        HistoryQuery {
            automation_id: id,
            limit,
            cursor,
        },
    )
    .await
    .map_err(command_error)
}

pub(super) fn set_global_paused(paused: bool) -> Result<(), String> {
    crate::services::config::update_config(|config| {
        config.heartbeat.global_paused = paused;
        Ok(())
    })
    .map_err(|_| "store_unavailable".into())
}

pub(super) fn ui_actor() -> AutomationActor {
    AutomationActor {
        origin: AutomationOrigin::UserInterface,
        session_or_channel_id: "desktop-ui".into(),
        current_automation_id: None,
    }
}

pub(super) fn command_error(error: AutomationError) -> String {
    error.code().to_string()
}

async fn validate_project(project_id: Option<&str>) -> Result<(), String> {
    if let Some(project_id) = project_id {
        crate::services::agent_local::directory_access::project_path(project_id)
            .await
            .map_err(|_| "invalid_project")?;
    }
    Ok(())
}
