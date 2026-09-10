use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const AUTOMATION_RUNTIME_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OccurrenceState {
    Pending,
    Running,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationOccurrence {
    pub id: Uuid,
    pub automation_id: Uuid,
    pub scheduled_for: DateTime<Utc>,
    pub state: OccurrenceState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub coalesced_count: Option<u32>,
    pub last_scheduled_for: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub result: Option<OccurrenceResult>,
}

impl AutomationOccurrence {
    pub fn pending(automation_id: Uuid, scheduled_for: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            automation_id,
            scheduled_for,
            state: OccurrenceState::Pending,
            created_at: scheduled_for,
            updated_at: scheduled_for,
            coalesced_count: Some(1),
            last_scheduled_for: Some(scheduled_for),
            started_at: None,
            result: None,
        }
    }

    pub fn missed(
        automation_id: Uuid,
        first: DateTime<Utc>,
        last: DateTime<Utc>,
        count: u32,
        decided_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            automation_id,
            scheduled_for: first,
            state: OccurrenceState::Terminal,
            created_at: decided_at,
            updated_at: decided_at,
            coalesced_count: None,
            last_scheduled_for: None,
            started_at: None,
            result: Some(OccurrenceResult {
                status: OccurrenceResultStatus::Missed,
                finished_at: decided_at,
                error_code: None,
                session_id: None,
                tokens: None,
                missed_count: Some(count),
                first_scheduled_for: Some(first),
                last_scheduled_for: Some(last),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccurrenceResult {
    pub status: OccurrenceResultStatus,
    pub finished_at: DateTime<Utc>,
    pub error_code: Option<String>,
    pub session_id: Option<String>,
    pub tokens: Option<u32>,
    pub missed_count: Option<u32>,
    pub first_scheduled_for: Option<DateTime<Utc>>,
    pub last_scheduled_for: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OccurrenceResultStatus {
    Ok,
    Error,
    Missed,
    Cancelled,
    Interrupted,
}

pub use super::runtime_recovery::recover_startup;
#[cfg(test)]
pub(crate) use super::runtime_recovery::recover_startup_at;
pub use super::runtime_scan::scan_and_advance;

pub async fn reserved_automation_ids() -> Result<HashSet<Uuid>, String> {
    reserved_automation_ids_at(&crate::services::paths::data_dir()).await
}

#[derive(Debug, Clone)]
pub struct AutomationRuntime {
    pub schema_version: u32,
    pub last_checked_at: DateTime<Utc>,
    pub occurrences: Vec<AutomationOccurrence>,
}

pub(crate) async fn read_at(root: &Path) -> Result<Option<AutomationRuntime>, String> {
    let _guard = super::store_lock().await;
    read_unlocked_at(root).await
}

pub(crate) async fn write_at(root: &Path, runtime: &AutomationRuntime) -> Result<(), String> {
    let _guard = super::store_lock().await;
    write_unlocked_at(root, runtime).await
}

pub(crate) async fn scan_and_advance_at(
    root: &Path,
    through: DateTime<Utc>,
    scan: impl FnOnce(DateTime<Utc>) -> Result<Vec<AutomationOccurrence>, String>,
) -> Result<Vec<Uuid>, String> {
    let _guard = super::store_lock().await;
    let mut runtime = read_unlocked_at(root).await?.unwrap_or(AutomationRuntime {
        schema_version: AUTOMATION_RUNTIME_SCHEMA_VERSION,
        last_checked_at: through,
        occurrences: Vec::new(),
    });
    if through < runtime.last_checked_at {
        return Err(runtime_error());
    }
    let added = scan(runtime.last_checked_at)?;
    let mut ids = Vec::with_capacity(added.len());
    for occurrence in added {
        ids.push(super::runtime_scan::apply(
            &mut runtime.occurrences,
            occurrence,
        )?);
    }
    if runtime.occurrences.len() > 128 {
        return Err(runtime_error());
    }
    runtime.last_checked_at = through;
    write_unlocked_at(root, &runtime).await?;
    Ok(ids)
}

pub(crate) async fn reserved_automation_ids_at(root: &Path) -> Result<HashSet<Uuid>, String> {
    let _guard = super::store_lock().await;
    reserved_automation_ids_unlocked_at(root).await
}

pub(crate) async fn reserved_automation_ids_unlocked_at(
    root: &Path,
) -> Result<HashSet<Uuid>, String> {
    Ok(read_unlocked_at(root)
        .await?
        .map(|runtime| {
            runtime
                .occurrences
                .into_iter()
                .map(|item| item.automation_id)
                .collect()
        })
        .unwrap_or_default())
}

pub(crate) async fn remove_pending_unlocked_at(
    root: &Path,
    automation_id: Uuid,
) -> Result<(), String> {
    let Some(mut runtime) = read_unlocked_at(root).await? else {
        return Ok(());
    };
    let before = runtime.occurrences.len();
    runtime.occurrences.retain(|item| {
        item.automation_id != automation_id || item.state != OccurrenceState::Pending
    });
    if runtime.occurrences.len() != before {
        write_unlocked_at(root, &runtime).await?;
    }
    Ok(())
}

pub(crate) async fn read_unlocked_at(root: &Path) -> Result<Option<AutomationRuntime>, String> {
    let path = path(root);
    let bytes =
        match crate::services::private_store::read_bounded_regular_async(path, 2 * 1024 * 1024)
            .await?
        {
            crate::services::private_store::BoundedFile::Missing => return Ok(None),
            crate::services::private_store::BoundedFile::Content(bytes) => bytes,
        };
    let runtime = super::runtime_wire::decode(&bytes)?;
    if let Some(runtime) = &runtime {
        super::runtime_validation::validate(runtime)?;
    }
    Ok(runtime)
}

pub(crate) async fn write_unlocked_at(
    root: &Path,
    runtime: &AutomationRuntime,
) -> Result<(), String> {
    super::runtime_validation::validate(runtime)?;
    let bytes = super::runtime_wire::encode(runtime)?;
    crate::services::private_store::atomic_write_async(path(root), bytes)
        .await
        .map_err(|_| runtime_error())
}

fn path(root: &Path) -> PathBuf {
    root.join("automation-runtime.json")
}

pub(super) fn runtime_error() -> String {
    "AUTOMATION_RUNTIME_UNAVAILABLE".into()
}
