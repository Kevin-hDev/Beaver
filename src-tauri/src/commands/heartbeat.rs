#[path = "heartbeat_contract.rs"]
mod contract;
#[path = "heartbeat_migration.rs"]
mod migration;
#[path = "heartbeat_store.rs"]
mod store;

use crate::models::HeartbeatConfig;
use crate::services::automations::{AutomationDetail, AutomationSummary, HistoryPage};
use crate::services::scheduler::Scheduler;
pub use contract::{
    ConflictDecision, CreateWakeupInput, MigrationStatusView, ScheduleInput, UpdateWakeupInput,
};
use tauri::State;
use uuid::Uuid;

#[cfg(test)]
use crate::commands::heartbeat_validation as validation;

#[cfg(test)]
#[path = "heartbeat_tests.rs"]
mod tests;

#[tauri::command]
pub async fn list_wakeups() -> Result<Vec<AutomationSummary>, String> {
    store::list().await
}

#[tauri::command]
pub async fn get_wakeup(automation_id: Uuid) -> Result<AutomationDetail, String> {
    store::get(automation_id).await
}

#[tauri::command]
pub async fn create_wakeup(
    input: CreateWakeupInput,
    scheduler: State<'_, Scheduler>,
) -> Result<AutomationDetail, String> {
    let result = store::create(input).await?;
    scheduler.notify_config_changed();
    Ok(result)
}

#[tauri::command]
pub async fn update_wakeup(
    input: UpdateWakeupInput,
    scheduler: State<'_, Scheduler>,
) -> Result<AutomationDetail, String> {
    let result = store::update(input).await?;
    scheduler.notify_config_changed();
    Ok(result)
}

#[tauri::command]
pub async fn delete_wakeup(id: Uuid, scheduler: State<'_, Scheduler>) -> Result<(), String> {
    store::delete(id).await?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub async fn set_wakeup_active(
    id: Uuid,
    active: bool,
    scheduler: State<'_, Scheduler>,
) -> Result<AutomationDetail, String> {
    if crate::services::config::read_config()
        .map_err(|_| "store_unavailable")?
        .heartbeat
        .global_paused
    {
        return Err("globally_paused".into());
    }
    let result = store::set_active(id, active).await?;
    scheduler.notify_config_changed();
    Ok(result)
}

#[tauri::command]
pub fn set_global_paused(paused: bool, scheduler: State<'_, Scheduler>) -> Result<(), String> {
    store::set_global_paused(paused)?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub fn get_heartbeat_config() -> Result<HeartbeatConfig, String> {
    crate::services::config::read_config()
        .map(|config| config.heartbeat)
        .map_err(|_| "store_unavailable".into())
}

#[tauri::command]
pub async fn list_wakeup_runs(
    wakeup_id: Uuid,
    limit: Option<usize>,
    cursor: Option<String>,
) -> Result<HistoryPage, String> {
    store::history(wakeup_id, limit, cursor).await
}

#[tauri::command]
pub async fn reconcile_automation_migration(
    timezone: Option<String>,
) -> Result<MigrationStatusView, String> {
    let detected = timezone.or_else(|| iana_time_zone::get_timezone().ok());
    migration::status_at(&crate::services::paths::data_dir(), detected.as_deref()).await
}

#[tauri::command]
pub async fn resolve_automation_migration_conflict(
    legacy_id: String,
    decision: ConflictDecision,
    timezone: Option<String>,
    scheduler: State<'_, Scheduler>,
) -> Result<Option<Uuid>, String> {
    let result = migration::resolve_at(
        &crate::services::paths::data_dir(),
        legacy_id,
        decision,
        timezone,
    )
    .await?;
    scheduler.notify_config_changed();
    Ok(result)
}

#[cfg(test)]
use migration::{resolve_at as resolve_conflict_at, status_at as migration_status_at};
#[cfg(test)]
use store::{
    command_error, create as create_wakeup_inner, delete as delete_wakeup_inner,
    get as get_wakeup_inner, history as list_wakeup_runs_inner, list as list_wakeups_inner,
    set_global_paused as set_global_paused_inner, update as update_wakeup_inner,
};
