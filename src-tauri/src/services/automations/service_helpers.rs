use super::*;
use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

pub(super) async fn find(root: &Path, id: Uuid) -> Result<AutomationDefinition, AutomationError> {
    super::store::read_all_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?
        .into_iter()
        .find(|item| item.id == id)
        .ok_or(AutomationError::NotFound)
}

pub(super) fn detail(
    definition: AutomationDefinition,
    now: DateTime<Utc>,
) -> Result<AutomationDetail, AutomationError> {
    let next_fire_at = crate::services::scheduler::next_fire::next_fire_at(&definition, now)
        .map_err(|_| AutomationError::InvalidSchedule)?
        .map(|next| next.at);
    Ok(AutomationDetail {
        definition,
        next_fire_at,
    })
}

pub(super) fn summary(definition: AutomationDefinition, now: DateTime<Utc>) -> AutomationSummary {
    summary_with_state(definition, now, false, false, None)
}

pub(super) fn summary_with_state(
    definition: AutomationDefinition,
    now: DateTime<Utc>,
    paused_by_global: bool,
    running: bool,
    last_run: Option<AutomationLastRun>,
) -> AutomationSummary {
    let next_fire_at = crate::services::scheduler::next_fire::next_fire_at(&definition, now)
        .ok()
        .flatten()
        .map(|next| next.at);
    AutomationSummary {
        id: definition.id,
        revision: definition.revision,
        name: definition.name,
        provider: definition.provider,
        model: definition.model,
        target: definition.target,
        schedule: definition.schedule,
        status: definition.status,
        running,
        paused_by_global,
        next_fire_at,
        last_run,
    }
}

pub(super) fn new_definition(
    actor: &AutomationActor,
    input: CreateAutomation,
    now: DateTime<Utc>,
) -> AutomationDefinition {
    let anchor_at =
        matches!(input.schedule, AutomationSchedule::AfterCompletion { .. }).then_some(now);
    AutomationDefinition {
        id: Uuid::new_v4(),
        revision: 1,
        name: input.name,
        description: input.description,
        prompt: input.prompt,
        creator_session_id: matches!(actor.origin, AutomationOrigin::Session)
            .then(|| actor.session_or_channel_id.clone()),
        target: input.target,
        provider: input.provider,
        model: input.model,
        schedule: input.schedule,
        status: input.status,
        created_at: now,
        anchor_at,
    }
}

pub(super) fn apply_patch(
    current: &mut AutomationDefinition,
    patch: UpdateAutomation,
    now: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let was_active = current.status == AutomationStatus::Active;
    let was_after_completion =
        matches!(current.schedule, AutomationSchedule::AfterCompletion { .. });
    let schedule_changed = patch.schedule.is_some();
    if let Some(name) = patch.name {
        current.name = name;
    }
    if let Some(description) = patch.description {
        current.description = description;
    }
    if let Some(prompt) = patch.prompt {
        current.prompt = prompt;
    }
    if let Some(model) = patch.model {
        current.model = model;
    }
    if let Some(schedule) = patch.schedule {
        current.schedule = schedule;
    }
    if let Some(status) = patch.status {
        current.status = status;
    }
    current.anchor_at = match current.schedule {
        AutomationSchedule::AfterCompletion { .. }
            if schedule_changed
                || !was_after_completion
                || (!was_active && current.status == AutomationStatus::Active) =>
        {
            Some(now)
        }
        AutomationSchedule::AfterCompletion { .. } => current.anchor_at,
        _ => None,
    };
    current.revision = current.revision.saturating_add(1);
    super::validation::validate_definition(current)
}

pub(super) fn validate_resume_actor(
    actor: &AutomationActor,
    target: &AutomationTarget,
) -> Result<(), AutomationError> {
    if let AutomationTarget::ResumeSession { session_id } = target {
        if actor.origin != AutomationOrigin::Session || session_id != &actor.session_or_channel_id {
            return Err(AutomationError::InvalidInput);
        }
    }
    Ok(())
}

pub(super) fn changed_fields(patch: &UpdateAutomation) -> Vec<String> {
    let mut fields = Vec::new();
    for (present, name) in [
        (patch.name.is_some(), "name"),
        (patch.description.is_some(), "description"),
        (patch.prompt.is_some(), "prompt"),
        (patch.model.is_some(), "model"),
        (patch.schedule.is_some(), "schedule"),
        (patch.status.is_some(), "status"),
    ] {
        if present {
            fields.push(name.to_string());
        }
    }
    fields
}
