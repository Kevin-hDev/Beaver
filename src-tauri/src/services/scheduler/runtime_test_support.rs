use crate::models::AutomationDefinition;
use crate::services::automations::{AutomationError, AutomationRuntime};
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

pub(crate) use super::runtime::AutomationRunResult;
pub(crate) use crate::services::automations::RuntimeAdmission as Admission;

pub(crate) async fn admit_due_at(
    root: &Path,
    definition: &AutomationDefinition,
    scheduled_for: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<Admission, AutomationError> {
    crate::services::automations::admit_runtime_at(root, definition, scheduled_for, now).await
}

pub(crate) async fn mark_running_at(
    root: &Path,
    id: Uuid,
    started_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    crate::services::automations::mark_runtime_running_at(root, id, started_at).await
}

pub(crate) async fn mark_terminal_at(
    root: &Path,
    id: Uuid,
    result: AutomationRunResult,
) -> Result<(), AutomationError> {
    crate::services::automations::mark_runtime_terminal_at(root, id, result).await
}

pub(crate) async fn runtime_at(root: &Path) -> Result<AutomationRuntime, AutomationError> {
    crate::services::automations::read_runtime_at(root).await
}

pub(crate) use super::runtime_publish::publish_terminal_at;
