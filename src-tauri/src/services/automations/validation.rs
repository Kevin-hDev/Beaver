use super::{AutomationError, CreateAutomation, UpdateAutomation};
use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use crate::services::llm;
use chrono::{DateTime, Utc};

pub(super) async fn validate_create(
    input: &CreateAutomation,
    now: DateTime<Utc>,
) -> Result<(), AutomationError> {
    if input.status == AutomationStatus::Completed {
        return Err(AutomationError::InvalidInput);
    }
    super::text_validation::validate_single_line_text(&input.name, 120)?;
    super::text_validation::validate_optional_multiline_text(input.description.as_deref(), 300)?;
    super::text_validation::validate_multiline_text(&input.prompt, 12_000)?;
    validate_target(&input.target)?;
    validate_model(&input.provider, &input.model).await?;
    validate_schedule_at(&input.schedule, now)
}

pub(super) async fn validate_update(
    current: &AutomationDefinition,
    patch: &UpdateAutomation,
    now: DateTime<Utc>,
) -> Result<(), AutomationError> {
    validate_update_locked(current, patch)?;
    if let Some(model) = &patch.model {
        validate_model(&current.provider, model).await?;
    }
    if let Some(schedule) = &patch.schedule {
        validate_schedule_at(schedule, now)?;
    }
    Ok(())
}

pub(super) fn validate_update_locked(
    current: &AutomationDefinition,
    patch: &UpdateAutomation,
) -> Result<(), AutomationError> {
    if patch.target.is_some() || patch.provider.is_some() || patch.creator_session_id.is_some() {
        return Err(AutomationError::ImmutableField);
    }
    if patch.status == Some(AutomationStatus::Completed) {
        return Err(AutomationError::InvalidInput);
    }
    if current.status == AutomationStatus::Completed
        && patch.schedule.is_some()
        && !matches!(
            patch.status,
            Some(AutomationStatus::Active | AutomationStatus::Disabled)
        )
    {
        return Err(AutomationError::InvalidSchedule);
    }
    if let Some(name) = &patch.name {
        super::text_validation::validate_single_line_text(name, 120)?;
    }
    if let Some(description) = &patch.description {
        super::text_validation::validate_optional_multiline_text(description.as_deref(), 300)?;
    }
    if let Some(prompt) = &patch.prompt {
        super::text_validation::validate_multiline_text(prompt, 12_000)?;
    }
    if let Some(schedule) = &patch.schedule {
        validate_schedule(schedule)?;
    }
    Ok(())
}

pub(super) fn validate_definition(
    definition: &AutomationDefinition,
) -> Result<(), AutomationError> {
    super::text_validation::validate_single_line_text(&definition.name, 120)?;
    super::text_validation::validate_optional_multiline_text(
        definition.description.as_deref(),
        300,
    )?;
    super::text_validation::validate_multiline_text(&definition.prompt, 12_000)?;
    validate_target(&definition.target)?;
    validate_schedule(&definition.schedule)?;
    match definition.schedule {
        AutomationSchedule::AfterCompletion { .. } if definition.anchor_at.is_some() => Ok(()),
        AutomationSchedule::AfterCompletion { .. } => Err(AutomationError::InvalidSchedule),
        _ if definition.anchor_at.is_none() => Ok(()),
        _ => Err(AutomationError::InvalidSchedule),
    }
}

async fn validate_model(provider: &str, model: &str) -> Result<(), AutomationError> {
    if llm::route::canonical_provider_id(provider) != provider
        || !llm::stream_dispatch::is_available(
            provider,
            llm::stream_dispatch::InvocationKind::Interactive,
            llm::request_purpose::RequestPurpose::Automation,
        )
    {
        return Err(AutomationError::ProviderUnavailable);
    }
    let info =
        llm::runtime_models::lookup(provider, model).or_else(|| local_model(provider, model));
    let info = match info {
        Some(info) => info,
        None => llm::model_catalog::list_models_for(provider)
            .await
            .map_err(|_| AutomationError::ModelUnavailable)?
            .into_iter()
            .find(|item| item.id == model)
            .ok_or(AutomationError::ModelUnavailable)?,
    };
    if !info.supports_tools {
        return Err(AutomationError::ModelToolsUnsupported);
    }
    Ok(())
}

fn local_model(provider: &str, model: &str) -> Option<llm::types::ModelInfo> {
    if provider == crate::services::codex_client::PROVIDER_ID {
        return crate::services::codex_client::model_catalog::fallback_models()
            .into_iter()
            .find(|item| item.id == model);
    }
    None
}

pub(crate) fn validate_schedule(schedule: &AutomationSchedule) -> Result<(), AutomationError> {
    match schedule {
        AutomationSchedule::Cron {
            expression,
            timezone,
        } => {
            let probe = schedule_probe(AutomationSchedule::Cron {
                expression: expression.clone(),
                timezone: *timezone,
            });
            super::next_fire::next_fire_at(&probe, Utc::now())
                .map(|_| ())
                .map_err(|_| AutomationError::InvalidSchedule)
        }
        AutomationSchedule::AfterCompletion { delay_minutes }
            if !(1..=525_600).contains(delay_minutes) =>
        {
            Err(AutomationError::InvalidSchedule)
        }
        _ => Ok(()),
    }
}

fn validate_schedule_at(
    schedule: &AutomationSchedule,
    now: DateTime<Utc>,
) -> Result<(), AutomationError> {
    validate_schedule(schedule)?;
    if let AutomationSchedule::Once { .. } = schedule {
        let probe = schedule_probe(schedule.clone());
        if super::next_fire::next_fire_at(&probe, now)
            .map_err(|_| AutomationError::InvalidSchedule)?
            .is_none()
        {
            return Err(AutomationError::InvalidSchedule);
        }
    }
    Ok(())
}

fn schedule_probe(schedule: AutomationSchedule) -> AutomationDefinition {
    AutomationDefinition {
        id: uuid::Uuid::nil(),
        revision: 1,
        name: "probe".into(),
        description: None,
        prompt: "probe".into(),
        creator_session_id: None,
        target: AutomationTarget::NewSession { project_id: None },
        provider: "probe".into(),
        model: "probe".into(),
        schedule,
        status: AutomationStatus::Active,
        created_at: Utc::now(),
        anchor_at: None,
    }
}

fn validate_target(target: &AutomationTarget) -> Result<(), AutomationError> {
    match target {
        AutomationTarget::ResumeSession { session_id } => {
            super::text_validation::validate_single_line_text(session_id, 128)
        }
        AutomationTarget::NewSession { project_id } => {
            super::text_validation::validate_optional_single_line_text(project_id.as_deref(), 128)
        }
    }
}
