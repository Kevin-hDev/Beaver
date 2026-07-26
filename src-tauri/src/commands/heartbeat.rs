use crate::commands::heartbeat_validation as validation;
use crate::models::{
    HeartbeatConfig, ScheduledWakeup, WakeupRun, WakeupSchedule, WakeupStatusSummary,
};
use crate::services::config as cfg;
use crate::services::scheduler::{log, next_fire::next_fire_at, Scheduler};
use chrono::Local;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateWakeupInput {
    pub name: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub schedule: WakeupSchedule,
    #[serde(default)]
    pub description: String,
}

#[tauri::command]
pub fn list_wakeups() -> Result<Vec<ScheduledWakeup>, String> {
    Ok(cfg::read_config()?.scheduled_wakeups)
}

#[tauri::command]
pub fn create_wakeup(
    input: CreateWakeupInput,
    scheduler: State<'_, Scheduler>,
) -> Result<ScheduledWakeup, String> {
    validation::validate_input(
        &input.provider,
        &input.name,
        &input.model,
        &input.prompt,
        &input.description,
        &input.schedule,
        true,
    )?;

    let mut config = cfg::read_config()?;
    validation::validate_capacity(config.scheduled_wakeups.len())?;
    let globally_paused = config.heartbeat.global_paused;
    let wakeup = ScheduledWakeup {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        model: input.model,
        provider: input.provider,
        prompt: input.prompt,
        schedule: input.schedule,
        description: input.description,
        active: !globally_paused,
        paused_by_global: globally_paused,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    config.scheduled_wakeups.push(wakeup.clone());
    cfg::write_config(&config)?;
    scheduler.notify_config_changed();
    Ok(wakeup)
}

#[tauri::command]
pub fn update_wakeup(
    mut wakeup: ScheduledWakeup,
    scheduler: State<'_, Scheduler>,
) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    if config.heartbeat.global_paused && wakeup.active {
        wakeup.active = false;
        wakeup.paused_by_global = true;
    }
    validation::validate_wakeup(&wakeup)?;

    let idx = config
        .scheduled_wakeups
        .iter()
        .position(|w| w.id == wakeup.id)
        .ok_or_else(|| "Réveil introuvable".to_string())?;

    config.scheduled_wakeups[idx] = wakeup;
    cfg::write_config(&config)?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub fn delete_wakeup(id: String, scheduler: State<'_, Scheduler>) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    config.scheduled_wakeups.retain(|w| w.id != id);
    cfg::write_config(&config)?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub fn set_wakeup_active(
    id: String,
    active: bool,
    scheduler: State<'_, Scheduler>,
) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    if config.heartbeat.global_paused {
        return Err("Réveils en veille".into());
    }

    let w = config
        .scheduled_wakeups
        .iter_mut()
        .find(|w| w.id == id)
        .ok_or_else(|| "Réveil introuvable".to_string())?;
    w.active = active;
    w.paused_by_global = false;
    validation::validate_wakeup(w)?;

    cfg::write_config(&config)?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub fn set_global_paused(paused: bool, scheduler: State<'_, Scheduler>) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    for w in &mut config.scheduled_wakeups {
        if paused && w.active {
            w.paused_by_global = true;
            w.active = false;
        } else if !paused && w.paused_by_global {
            w.active = true;
            w.paused_by_global = false;
        }
    }

    config.heartbeat.global_paused = paused;
    cfg::write_config(&config)?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub fn get_heartbeat_config() -> Result<HeartbeatConfig, String> {
    Ok(cfg::read_config()?.heartbeat)
}

#[tauri::command]
pub async fn list_wakeup_runs(wakeup_id: Option<String>) -> Result<Vec<WakeupRun>, String> {
    log::list_runs(wakeup_id.as_deref()).await
}

#[tauri::command]
pub async fn get_wakeup_status_summaries() -> Result<Vec<WakeupStatusSummary>, String> {
    let config = cfg::read_config()?;
    let runs = log::list_runs(None).await?;
    let now = Local::now();
    let summaries = config
        .scheduled_wakeups
        .iter()
        .map(|w| {
            let next_fire_at = if config.heartbeat.global_paused || !w.active || w.paused_by_global
            {
                None
            } else {
                next_fire_at(&w.schedule, now).map(|dt| dt.to_rfc3339())
            };
            let last_run = runs.iter().find(|r| r.wakeup_id == w.id).cloned();
            WakeupStatusSummary {
                wakeup_id: w.id.clone(),
                next_fire_at,
                last_run,
            }
        })
        .collect();
    Ok(summaries)
}
