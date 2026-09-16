use crate::models::AutomationSchedule;
use crate::services::automations::AutomationError;
use serde::Deserialize;
use serde_json::{json, Value};

use super::core_bridge::ExtensionBridgeError;

pub(super) fn public_automation(value: crate::models::AutomationDefinition) -> Value {
    json!({
        "id": value.id,
        "revision": value.revision,
        "name": value.name,
        "description": value.description,
        "prompt": value.prompt,
        "schedule": value.schedule,
        "active": value.status == crate::models::AutomationStatus::Active,
    })
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum PublicSchedule {
    Once { local_datetime: String, timezone: String },
    Cron { expression: String, timezone: String },
    AfterCompletion { delay_minutes: u32 },
}

pub(super) fn schedule(value: Option<&Value>) -> Result<AutomationSchedule, ExtensionBridgeError> {
    let value = serde_json::from_value::<PublicSchedule>(
        value.cloned().ok_or(ExtensionBridgeError::Denied)?,
    )
    .map_err(|_| ExtensionBridgeError::Denied)?;
    match value {
        PublicSchedule::Once { local_datetime, timezone } => Ok(AutomationSchedule::Once {
            local_datetime: local_datetime.parse().map_err(|_| ExtensionBridgeError::Denied)?,
            timezone: timezone.parse().map_err(|_| ExtensionBridgeError::Denied)?,
        }),
        PublicSchedule::Cron { expression, timezone } => Ok(AutomationSchedule::Cron {
            expression,
            timezone: timezone.parse().map_err(|_| ExtensionBridgeError::Denied)?,
        }),
        PublicSchedule::AfterCompletion { delay_minutes } => {
            Ok(AutomationSchedule::AfterCompletion { delay_minutes })
        }
    }
}

pub(super) fn id(params: &Value) -> Result<uuid::Uuid, ExtensionBridgeError> {
    required(params, "automationId")?
        .parse()
        .map_err(|_| ExtensionBridgeError::Denied)
}

pub(super) fn revision(params: &Value) -> Result<u64, ExtensionBridgeError> {
    params.get("revision").and_then(Value::as_u64).ok_or(ExtensionBridgeError::Denied)
}

pub(super) fn cursor(params: &Value) -> Result<usize, ExtensionBridgeError> {
    optional(params, "cursor")?
        .map_or(Ok(0), |value| value.parse().map_err(|_| ExtensionBridgeError::Denied))
}

pub(super) fn required<'a>(
    params: &'a Value,
    key: &str,
) -> Result<&'a str, ExtensionBridgeError> {
    optional(params, key)?.ok_or(ExtensionBridgeError::Denied)
}

pub(super) fn optional<'a>(
    params: &'a Value,
    key: &str,
) -> Result<Option<&'a str>, ExtensionBridgeError> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.is_empty() => Ok(Some(value)),
        _ => Err(ExtensionBridgeError::Denied),
    }
}

pub(super) fn map_error(error: AutomationError) -> ExtensionBridgeError {
    match error {
        AutomationError::NotFound => ExtensionBridgeError::Backend("core_automation_not_found"),
        AutomationError::RevisionConflict => {
            ExtensionBridgeError::Backend("core_automation_revision_conflict")
        }
        AutomationError::GloballyPaused => {
            ExtensionBridgeError::Backend("core_automation_globally_paused")
        }
        AutomationError::ConsentRequired => {
            ExtensionBridgeError::Backend("core_automation_consent_required")
        }
        AutomationError::CapacityReached => ExtensionBridgeError::Backend("core_saturated"),
        AutomationError::InvalidInput
        | AutomationError::ImmutableField
        | AutomationError::InvalidSchedule => ExtensionBridgeError::Denied,
        _ => ExtensionBridgeError::Failed,
    }
}
