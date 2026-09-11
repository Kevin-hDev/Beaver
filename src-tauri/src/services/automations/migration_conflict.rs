use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const MAX_CONFIG_BYTES: u64 = 2 * 1024 * 1024;

pub enum ConflictResolution {
    RemoveHistorical,
    ImportAsNew { timezone: Tz },
}

pub async fn resolve_audited_at(
    root: &Path,
    actor: &super::AutomationActor,
    legacy_id: &str,
    resolution: ConflictResolution,
) -> Result<Option<Uuid>, String> {
    let target_id = Uuid::parse_str(legacy_id).ok();
    let operation = super::audit_store::begin(
        root,
        actor,
        "resolve_migration_conflict",
        target_id,
        vec!["resolution".into()],
    )
    .await
    .map_err(|_| error())?;
    let result = resolve_at(root, legacy_id, resolution)
        .await
        .map_err(|_| super::AutomationError::StoreUnavailable);
    super::audit_store::complete(
        root,
        actor,
        operation,
        "resolve_migration_conflict",
        target_id,
        result,
        None,
    )
    .await
    .map_err(|_| error())
}

#[derive(Serialize, Deserialize)]
struct ResolutionJournal {
    legacy_id: String,
    raw: Value,
    new_id: Option<Uuid>,
    timezone: Option<Tz>,
}

pub async fn resolve_at(
    root: &Path,
    legacy_id: &str,
    resolution: ConflictResolution,
) -> Result<Option<Uuid>, String> {
    if legacy_id.is_empty() || legacy_id.len() > 128 || legacy_id.chars().any(char::is_control) {
        return Err(error());
    }
    let _guard = super::store_lock().await;
    resume_at(root).await?;
    let config = read_config(root).await?;
    let raw = find_entry(&config, legacy_id).ok_or_else(error)?;
    let (new_id, timezone) = match resolution {
        ConflictResolution::RemoveHistorical => (None, None),
        ConflictResolution::ImportAsNew { timezone } => (Some(Uuid::new_v4()), Some(timezone)),
    };
    let journal = ResolutionJournal {
        legacy_id: legacy_id.to_string(),
        raw,
        new_id,
        timezone,
    };
    write_journal(root, &journal).await?;
    apply(root, &journal).await?;
    remove_journal(root)?;
    Ok(new_id)
}

pub(super) async fn resume_at(root: &Path) -> Result<(), String> {
    let path = journal_path(root);
    let bytes =
        match crate::services::private_store::read_bounded_regular_async(path, MAX_CONFIG_BYTES)
            .await
            .map_err(|_| error())?
        {
            crate::services::private_store::BoundedFile::Missing => return Ok(()),
            crate::services::private_store::BoundedFile::Content(bytes) => bytes,
        };
    let journal: ResolutionJournal = serde_json::from_slice(&bytes).map_err(|_| error())?;
    apply(root, &journal).await?;
    remove_journal(root)
}

async fn apply(root: &Path, journal: &ResolutionJournal) -> Result<(), String> {
    if let (Some(new_id), Some(timezone)) = (journal.new_id, journal.timezone) {
        let mut definition = super::migration_convert::convert(&journal.raw, timezone)?;
        definition.id = new_id;
        let mut definitions = super::store::read_all_unlocked_at(root).await?;
        if !definitions.iter().any(|item| item.id == new_id) {
            definitions.push(definition);
            super::store::write_definitions_unlocked_at(root, definitions).await?;
        }
    }
    let mut config = read_config(root).await?;
    let Some(entries) = config
        .get_mut("scheduled_wakeups")
        .and_then(Value::as_array_mut)
    else {
        return Ok(());
    };
    entries.retain(|entry| entry != &journal.raw);
    if entries.is_empty() {
        config
            .as_object_mut()
            .ok_or_else(error)?
            .remove("scheduled_wakeups");
    }
    write(
        root.join("config.json"),
        serde_json::to_vec_pretty(&config).map_err(|_| error())?,
    )
    .await?;
    if let Ok(id) = Uuid::parse_str(&journal.legacy_id) {
        super::release_retired_unlocked_at(root, id).await?;
    }
    Ok(())
}

async fn read_config(root: &Path) -> Result<Value, String> {
    let bytes = match crate::services::private_store::read_bounded_regular_async(
        root.join("config.json"),
        MAX_CONFIG_BYTES,
    )
    .await
    .map_err(|_| error())?
    {
        crate::services::private_store::BoundedFile::Missing => return Err(error()),
        crate::services::private_store::BoundedFile::Content(bytes) => bytes,
    };
    serde_json::from_slice(&bytes).map_err(|_| error())
}

fn find_entry(config: &Value, legacy_id: &str) -> Option<Value> {
    config
        .get("scheduled_wakeups")?
        .as_array()?
        .iter()
        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(legacy_id))
        .cloned()
}

async fn write_journal(root: &Path, journal: &ResolutionJournal) -> Result<(), String> {
    write(
        journal_path(root),
        serde_json::to_vec_pretty(journal).map_err(|_| error())?,
    )
    .await
}

async fn write(path: PathBuf, bytes: Vec<u8>) -> Result<(), String> {
    crate::services::private_store::atomic_write_async(path, bytes)
        .await
        .map_err(|_| error())
}

fn remove_journal(root: &Path) -> Result<(), String> {
    std::fs::remove_file(journal_path(root)).map_err(|_| error())
}

fn journal_path(root: &Path) -> PathBuf {
    root.join(".automation-conflict-resolution.json")
}

fn error() -> String {
    "AUTOMATION_MIGRATION_UNAVAILABLE".into()
}
