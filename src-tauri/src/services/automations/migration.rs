use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

use super::migration_files::{cleanup, finish_interrupted_publication, stop, write};

const MAX_CONFIG_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", content = "conflicts", rename_all = "snake_case")]
pub enum AutomationMigrationStatus {
    Ready,
    NeedsTimezone,
    Conflicts(Vec<MigrationConflict>),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct MigrationConflict {
    pub legacy_id: String,
}

pub async fn migrate_legacy(
    root: &Path,
    timezone_override: Option<Tz>,
) -> Result<AutomationMigrationStatus, String> {
    migrate_legacy_at(root, timezone_override).await
}

pub(crate) async fn migrate_legacy_at(
    root: &Path,
    timezone_override: Option<Tz>,
) -> Result<AutomationMigrationStatus, String> {
    migrate_legacy_impl(root, timezone_override, None).await
}

#[cfg(test)]
pub(crate) async fn migrate_legacy_stopping_after(
    root: &Path,
    timezone_override: Option<Tz>,
    boundary: u8,
) -> Result<AutomationMigrationStatus, String> {
    migrate_legacy_impl(root, timezone_override, Some(boundary)).await
}

async fn migrate_legacy_impl(
    root: &Path,
    timezone_override: Option<Tz>,
    stop_after: Option<u8>,
) -> Result<AutomationMigrationStatus, String> {
    let _guard = super::store_lock().await;
    super::migration_conflict::resume_at(root).await?;
    let config_path = root.join("config.json");
    let bytes = match crate::services::private_store::read_bounded_regular_async(
        config_path.clone(),
        MAX_CONFIG_BYTES,
    )
    .await
    .map_err(|_| migration_error())?
    {
        crate::services::private_store::BoundedFile::Missing => return Ok(cleanup(root)),
        crate::services::private_store::BoundedFile::Content(bytes) => bytes,
    };
    let mut config: Value = serde_json::from_slice(&bytes).map_err(|_| migration_error())?;
    let entries = legacy_entries(&config)?;
    if entries.is_empty() {
        finish_interrupted_publication(root).await?;
        return Ok(cleanup(root));
    }
    let Some(timezone) = timezone_override else {
        return Ok(AutomationMigrationStatus::NeedsTimezone);
    };
    reconcile(
        root,
        &config_path,
        &bytes,
        &mut config,
        entries,
        timezone,
        stop_after,
    )
    .await
}

async fn reconcile(
    root: &Path,
    config_path: &Path,
    original: &[u8],
    config: &mut Value,
    entries: Vec<Value>,
    timezone: Tz,
    stop_after: Option<u8>,
) -> Result<AutomationMigrationStatus, String> {
    write(
        root.join("config.pre-automations-v1.json"),
        original.to_vec(),
    )
    .await?;
    stop(1, stop_after)?;
    let mut definitions = super::store::read_all_unlocked_at(root).await?;
    let reserved = super::reserved_unlocked_at(root).await?;
    let mut retained = Vec::new();
    let mut conflicts = Vec::new();
    for raw in entries {
        let converted = super::migration_convert::convert(&raw, timezone);
        match converted {
            Ok(definition) if reserved.contains(&definition.id) => {
                conflict(&raw, &mut retained, &mut conflicts)
            }
            Ok(definition) => match definitions.iter().find(|item| item.id == definition.id) {
                Some(existing) if existing == &definition => {}
                Some(_) => conflict(&raw, &mut retained, &mut conflicts),
                None => definitions.push(definition),
            },
            Err(_) => conflict(&raw, &mut retained, &mut conflicts),
        }
    }
    super::store::write_definitions_unlocked_at(root, definitions).await?;
    super::store::read_all_unlocked_at(root).await?;
    stop(2, stop_after)?;
    ensure_runtime(root).await?;
    stop(3, stop_after)?;
    set_legacy_entries(config, retained)?;
    write(
        config_path.to_path_buf(),
        serde_json::to_vec_pretty(config).map_err(|_| migration_error())?,
    )
    .await?;
    stop(4, stop_after)?;
    write(root.join(".automations-v1-migrated"), b"pending".to_vec()).await?;
    stop(5, stop_after)?;
    if conflicts.is_empty() {
        Ok(AutomationMigrationStatus::Ready)
    } else {
        Ok(AutomationMigrationStatus::Conflicts(conflicts))
    }
}

async fn ensure_runtime(root: &Path) -> Result<(), String> {
    if super::runtime_store::read_unlocked_at(root)
        .await?
        .is_some()
    {
        return Ok(());
    }
    let last_checked_at = legacy_last_checked(root).unwrap_or_else(Utc::now);
    super::runtime_store::write_unlocked_at(
        root,
        &super::runtime_store::AutomationRuntime {
            schema_version: super::runtime_store::AUTOMATION_RUNTIME_SCHEMA_VERSION,
            last_checked_at,
            occurrences: Vec::new(),
            retired_automation_ids: Vec::new(),
        },
    )
    .await?;
    super::runtime_store::read_unlocked_at(root).await?;
    Ok(())
}

fn legacy_last_checked(root: &Path) -> Option<DateTime<Utc>> {
    let bytes = std::fs::read(root.join("heartbeat-runtime.json")).ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    DateTime::parse_from_rfc3339(value.get("last_checked_at")?.as_str()?)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn legacy_entries(config: &Value) -> Result<Vec<Value>, String> {
    match config.get("scheduled_wakeups") {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(items)) if items.len() <= 64 => Ok(items.clone()),
        Some(_) => Err(migration_error()),
    }
}

fn set_legacy_entries(config: &mut Value, entries: Vec<Value>) -> Result<(), String> {
    let object = config.as_object_mut().ok_or_else(migration_error)?;
    if entries.is_empty() {
        object.remove("scheduled_wakeups");
    } else {
        object.insert("scheduled_wakeups".into(), Value::Array(entries));
    }
    Ok(())
}

fn conflict(raw: &Value, retained: &mut Vec<Value>, conflicts: &mut Vec<MigrationConflict>) {
    let legacy_id = raw
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    retained.push(raw.clone());
    conflicts.push(MigrationConflict { legacy_id });
}

pub fn acknowledge_successful_startup(root: &Path) -> Result<(), String> {
    super::migration_files::acknowledge_successful_startup(root)
}

pub(super) fn migration_error() -> String {
    "AUTOMATION_MIGRATION_UNAVAILABLE".into()
}
