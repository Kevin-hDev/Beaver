use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use crate::services::automations::{AutomationDetail, AutomationSummary, HistoryPage};
use serde_json::{json, Value};

pub(super) fn summaries(items: Vec<AutomationSummary>) -> Value {
    Value::Array(items.into_iter().map(summary).collect())
}

pub(super) fn detail(value: AutomationDetail) -> Value {
    let definition = value.definition;
    let mut data = common(&definition);
    let object = data.as_object_mut().expect("automation detail is an object");
    object.insert("description".into(), json!(definition.description));
    object.insert("prompt".into(), json!(definition.prompt));
    object.insert(
        "creator_session_id".into(),
        json!(definition.creator_session_id),
    );
    object.insert("created_at".into(), json!(definition.created_at.to_rfc3339()));
    object.insert("anchor_at".into(), json!(definition.anchor_at.map(|at| at.to_rfc3339())));
    object.insert(
        "next_fire_at".into(),
        json!(value.next_fire_at.map(|at| at.to_rfc3339())),
    );
    data
}

pub(super) fn history(page: HistoryPage) -> Value {
    json!({"items": page.entries, "next_cursor": page.next_cursor})
}

fn summary(value: AutomationSummary) -> Value {
    let status = if value.running {
        "running"
    } else if value.paused_by_global {
        "paused_by_global"
    } else {
        status(value.status)
    };
    let (target_mode, target_session_id, project_id) = target(&value.target);
    json!({
        "id": value.id,
        "revision": value.revision,
        "name": value.name,
        "provider": value.provider,
        "model": value.model,
        "target_mode": target_mode,
        "target_session_id": target_session_id,
        "project_id": project_id,
        "schedule": schedule(&value.schedule),
        "status": status,
        "next_fire_at": value.next_fire_at.map(|at| at.to_rfc3339()),
        "last_run": value.last_run.map(|run| json!({
            "status": run.status,
            "finished_at": run.finished_at,
        })),
    })
}

fn common(value: &AutomationDefinition) -> Value {
    let (target_mode, target_session_id, project_id) = target(&value.target);
    json!({
        "id": value.id,
        "revision": value.revision,
        "name": value.name,
        "provider": value.provider,
        "model": value.model,
        "target_mode": target_mode,
        "target_session_id": target_session_id,
        "project_id": project_id,
        "schedule": schedule(&value.schedule),
        "status": status(value.status),
    })
}

fn target(value: &AutomationTarget) -> (&'static str, Option<&str>, Option<&str>) {
    match value {
        AutomationTarget::NewSession { project_id } => {
            ("new_session", None, project_id.as_deref())
        }
        AutomationTarget::ResumeSession { session_id } => {
            ("resume_session", Some(session_id), None)
        }
    }
}

fn schedule(value: &AutomationSchedule) -> Value {
    match value {
        AutomationSchedule::Once {
            local_datetime,
            timezone,
        } => json!({
            "kind": "once",
            "local_datetime": local_datetime.format("%Y-%m-%dT%H:%M").to_string(),
            "timezone": timezone.name(),
        }),
        AutomationSchedule::Cron {
            expression,
            timezone,
        } => json!({"kind":"cron", "expression":expression, "timezone":timezone.name()}),
        AutomationSchedule::AfterCompletion { delay_minutes } => {
            json!({"kind":"after_completion", "delay_minutes":delay_minutes})
        }
    }
}

fn status(value: AutomationStatus) -> &'static str {
    match value {
        AutomationStatus::Active => "active",
        AutomationStatus::Disabled => "disabled",
        AutomationStatus::Completed => "completed",
    }
}
