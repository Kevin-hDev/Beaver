use super::tool_automation_validation as validation;
use crate::models::{ScheduledWakeup, WakeupSchedule};
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::{json, Value};
use std::path::Path;
use uuid::Uuid;

pub async fn execute(args: &Value, _working_dir: &Path, session_id: &str) -> ToolResult {
    match args["action"].as_str() {
        Some("list") => list(),
        Some("create") => create(args, session_id).await,
        Some("update") => update(args).await,
        Some("delete") => delete(args),
        _ => ToolResult::validation("automation_action_invalid", "Action d'automatisation invalide."),
    }
}

fn list() -> ToolResult {
    match crate::services::config::read_config() {
        Ok(config) => {
            let entries = config
                .scheduled_wakeups
                .into_iter()
                .map(|wakeup| public_value(&wakeup))
                .collect::<Vec<_>>();
            ToolResult::ok(serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".into()))
        }
        Err(_) => validation::internal_error(),
    }
}

async fn create(args: &Value, session_id: &str) -> ToolResult {
    let session = match super::session_store::get(session_id).await {
        Ok(session) => session,
        Err(_) => return validation::internal_error(),
    };
    let schedule = match validation::parse_schedule(args.get("schedule")) {
        Ok(value) => value, Err(result) => return result,
    };
    let wakeup = ScheduledWakeup {
        id: Uuid::new_v4().to_string(),
        name: validation::text(args, "name"),
        model: session.model,
        provider: session.provider,
        prompt: validation::text(args, "prompt"),
        schedule,
        description: validation::text(args, "description"),
        project_id: session.project_id,
        active: args["active"].as_bool().unwrap_or(true),
        paused_by_global: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    if let Err(error) = crate::commands::heartbeat_validation::validate_wakeup(&wakeup) {
        return ToolResult::validation("automation_invalid", error);
    }
    save_new(wakeup)
}

fn save_new(wakeup: ScheduledWakeup) -> ToolResult {
    let saved = crate::services::config::update_config(|config| {
        crate::commands::heartbeat_validation::validate_capacity(config.scheduled_wakeups.len())?;
        let mut wakeup = wakeup;
        if config.heartbeat.global_paused && wakeup.active {
            wakeup.active = false;
            wakeup.paused_by_global = true;
        }
        config.scheduled_wakeups.push(wakeup.clone());
        Ok(wakeup)
    });
    match saved {
        Ok(wakeup) => {
            crate::services::scheduler::notify_config_changed();
            ToolResult::ok(
                json!({"status":"created", "automation": public_value(&wakeup)}).to_string(),
            )
        }
        Err(error) => ToolResult::validation("automation_create_failed", error),
    }
}

async fn update(args: &Value) -> ToolResult {
    let id = validation::text(args, "id");
    let schedule = match optional_schedule(args) { Ok(value) => value, Err(result) => return result };
    update_saved(args, &id, schedule)
}

fn optional_schedule(args: &Value) -> Result<Option<WakeupSchedule>, ToolResult> {
    match args.get("schedule") {
        Some(value) if !value.is_null() => validation::parse_schedule(Some(value)).map(Some),
        _ => Ok(None),
    }
}

fn update_saved(
    args: &Value,
    id: &str,
    schedule: Option<WakeupSchedule>,
) -> ToolResult {
    let result = crate::services::config::update_config(|config| {
        let globally_paused = config.heartbeat.global_paused;
        let wakeup = config.scheduled_wakeups.iter_mut()
            .find(|wakeup| wakeup.id == id)
            .ok_or_else(|| "Automatisation introuvable".to_string())?;
        apply_optional(args, wakeup, schedule);
        if globally_paused && wakeup.active {
            wakeup.active = false;
            wakeup.paused_by_global = true;
        } else if !wakeup.active {
            wakeup.paused_by_global = false;
        }
        crate::commands::heartbeat_validation::validate_wakeup(wakeup)?;
        Ok(wakeup.clone())
    });
    match result {
        Ok(wakeup) => {
            crate::services::scheduler::notify_config_changed();
            ToolResult::ok(
                json!({"status":"updated", "automation": public_value(&wakeup)}).to_string(),
            )
        }
        Err(error) => ToolResult::validation("automation_update_failed", error),
    }
}

fn delete(args: &Value) -> ToolResult {
    if args["confirm"].as_bool() != Some(true) {
        return ToolResult::validation("automation_confirmation_required", "Confirmation requise.");
    }
    let id = validation::text(args, "id");
    let result = crate::services::config::update_config(|config| {
        let before = config.scheduled_wakeups.len();
        config.scheduled_wakeups.retain(|wakeup| wakeup.id != id);
        if config.scheduled_wakeups.len() == before { return Err("Automatisation introuvable".into()); }
        Ok(())
    });
    match result {
        Ok(()) => {
            crate::services::scheduler::notify_config_changed();
            ToolResult::ok(json!({"status":"deleted", "id": id}).to_string())
        }
        Err(error) => ToolResult::validation("automation_delete_failed", error),
    }
}

fn apply_optional(
    args: &Value,
    wakeup: &mut ScheduledWakeup,
    schedule: Option<WakeupSchedule>,
) {
    if let Some(value) = args["name"].as_str() { wakeup.name = value.to_string(); }
    if let Some(value) = args["description"].as_str() { wakeup.description = value.to_string(); }
    if let Some(value) = args["prompt"].as_str() { wakeup.prompt = value.to_string(); }
    if let Some(value) = args["active"].as_bool() { wakeup.active = value; }
    if let Some(value) = schedule { wakeup.schedule = value; }
}

pub(super) fn public_value(wakeup: &ScheduledWakeup) -> Value {
    json!({
        "id": wakeup.id,
        "name": wakeup.name,
        "description": wakeup.description,
        "prompt": wakeup.prompt,
        "schedule": wakeup.schedule,
        "active": wakeup.active,
        "project_id": wakeup.project_id,
    })
}
