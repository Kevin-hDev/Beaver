use crate::models::{
    AutomationDefinition, AutomationStatus, AutomationTarget, ScheduledWakeup, WakeupSchedule,
};
use crate::services::agent_local::session_store;
use crate::services::automations::{OccurrenceResult, OccurrenceResultStatus};
use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub async fn fire_automation(
    app: AppHandle,
    automation_id: Uuid,
    occurrence_id: Uuid,
    cancel: CancellationToken,
) {
    let Some(definition) = current_definition(automation_id).await else {
        return;
    };
    let session = match target_session(&definition).await {
        Ok(session) => session,
        Err(code) => {
            if super::runtime::mark_terminal(occurrence_id, error_result(code, None))
                .await
                .is_err()
            {
                return;
            }
            let _ = crate::services::automations::disable_missing_target(automation_id).await;
            if super::runtime::publish_terminal(occurrence_id)
                .await
                .is_err()
            {
                ::log::warn!("[scheduler] publication terminale différée");
            }
            return;
        }
    };
    let stream =
        match crate::commands::agent_chat_admission::admit_background_if_idle(&app, &session.id)
            .await
        {
            Ok(stream) => stream,
            Err(_) => {
                if session.created {
                    delete_empty_session(&session.id).await;
                }
                return;
            }
        };
    let generation = stream.generation;
    if super::runtime::mark_running(occurrence_id, Utc::now())
        .await
        .is_err()
    {
        let _ = crate::commands::agent_chat_streams::finish_active_stream(
            &app.state::<crate::ActiveStreams>(),
            &session.id,
            generation,
        )
        .await;
        if session.created {
            delete_empty_session(&session.id).await;
        }
        return;
    }
    let wakeup = legacy_adapter(&definition);
    let result = super::agentic::run(&app, &wakeup, &session.id, stream, cancel.clone()).await;
    if cancel.is_cancelled() {
        return;
    }
    match result {
        Ok(result) if result.has_text_result => {
            finish(
                occurrence_id,
                OccurrenceResult {
                    status: OccurrenceResultStatus::Ok,
                    finished_at: Utc::now(),
                    error_code: None,
                    session_id: Some(session.id.clone()),
                    tokens: Some(result.tokens),
                    missed_count: None,
                    first_scheduled_for: None,
                    last_scheduled_for: None,
                },
            )
            .await;
            let _ = app.emit(
                "wakeup-completed",
                serde_json::json!({"wakeup_id": automation_id, "session_id": session.id}),
            );
        }
        Ok(_) => finish_error(occurrence_id, "assistant_missing", Some(session.id)).await,
        Err(error) => {
            if session.created {
                delete_empty_session(&session.id).await;
            }
            finish_error(occurrence_id, error_code(&error), Some(session.id)).await;
            let _ = app.emit(
                "wakeup-failed",
                serde_json::json!({"wakeup_id": automation_id, "error": "Le réveil a échoué"}),
            );
        }
    }
}

async fn finish_error(occurrence_id: Uuid, code: &str, session_id: Option<String>) {
    finish(occurrence_id, error_result(code, session_id)).await;
}

fn error_result(code: &str, session_id: Option<String>) -> OccurrenceResult {
    OccurrenceResult {
        status: OccurrenceResultStatus::Error,
        finished_at: Utc::now(),
        error_code: Some(code.into()),
        session_id,
        tokens: None,
        missed_count: None,
        first_scheduled_for: None,
        last_scheduled_for: None,
    }
}

async fn finish(id: Uuid, result: OccurrenceResult) {
    if super::runtime::mark_terminal(id, result).await.is_ok()
        && super::runtime::publish_terminal(id).await.is_err()
    {
        ::log::warn!("[scheduler] publication terminale différée");
    }
}

async fn current_definition(id: Uuid) -> Option<AutomationDefinition> {
    crate::services::automations::read_all()
        .await
        .ok()?
        .into_iter()
        .find(|item| item.id == id && item.status == AutomationStatus::Active)
}

struct TargetSession {
    id: String,
    created: bool,
}

async fn target_session(definition: &AutomationDefinition) -> Result<TargetSession, &'static str> {
    match &definition.target {
        AutomationTarget::ResumeSession { session_id } => session_store::get(session_id)
            .await
            .map(|_| TargetSession {
                id: session_id.clone(),
                created: false,
            })
            .map_err(|_| "target_session_missing"),
        AutomationTarget::NewSession { project_id } => {
            create_session(definition, project_id.clone())
                .await
                .map(|id| TargetSession { id, created: true })
                .map_err(|_| "provider_unavailable")
        }
    }
}

async fn create_session(
    definition: &AutomationDefinition,
    project_id: Option<String>,
) -> Result<String, String> {
    let name = format!(
        "Automatisation • {} • {}",
        definition.name, definition.model
    );
    session_store::create_with_project(
        &name,
        &definition.model,
        &definition.provider,
        true,
        project_id,
    )
    .await
    .map(|session| session.id)
}

async fn delete_empty_session(session_id: &str) {
    let Ok(session) = session_store::get(session_id).await else {
        return;
    };
    if session.messages.is_empty() && session_store::delete_one(session_id).await.is_err() {
        ::log::warn!("empty_automation_cleanup_failed");
    }
}

fn legacy_adapter(definition: &AutomationDefinition) -> ScheduledWakeup {
    ScheduledWakeup {
        id: definition.id.to_string(),
        name: definition.name.clone(),
        model: definition.model.clone(),
        provider: definition.provider.clone(),
        prompt: definition.prompt.clone(),
        schedule: WakeupSchedule::Once {
            datetime: definition.created_at.to_rfc3339(),
        },
        description: definition.description.clone().unwrap_or_default(),
        project_id: match &definition.target {
            AutomationTarget::NewSession { project_id } => project_id.clone(),
            AutomationTarget::ResumeSession { .. } => None,
        },
        active: true,
        paused_by_global: false,
        created_at: definition.created_at.to_rfc3339(),
    }
}

fn error_code(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if lower.contains("auth") || lower.contains("401") || lower.contains("403") {
        "authentication_failed"
    } else if lower.contains("model") {
        "model_unavailable"
    } else if lower.contains("provider") || lower.contains("ollama") {
        "provider_unavailable"
    } else {
        "failed"
    }
}

#[cfg(test)]
#[path = "fire_tests.rs"]
mod tests;
