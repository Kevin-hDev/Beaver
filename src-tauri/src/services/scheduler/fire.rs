use crate::models::{AutomationDefinition, AutomationStatus, AutomationTarget};
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
    let admission =
        match super::extension_admission::admit(&definition, occurrence_id, &cancel).await {
            Ok(admission) => admission,
            Err(code) => {
                super::fire_result::reject_before_start(occurrence_id, automation_id, code).await;
                return;
            }
        };
    let cancel = admission.cancel.clone();
    let session = match target_session(&definition).await {
        Ok(session) => session,
        Err(code) => {
            if super::runtime::mark_terminal(
                occurrence_id,
                super::fire_result::error_result(code, None),
            )
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
    let actor_guard =
        match super::fire_actor::register(&session.id, &stream.request_id, automation_id).await {
            Ok(guard) => guard,
            Err(_) => {
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
        };
    let request_id = stream.request_id.clone();
    let result = super::agentic::run(
        &app,
        &definition,
        occurrence_id,
        &session.id,
        stream,
        cancel.clone(),
    )
    .await;
    drop(actor_guard);
    if cancel.is_cancelled() {
        super::fire_result::finish(
            occurrence_id,
            automation_id,
            &session.id,
            &request_id,
            super::fire_result::cancelled_result(Some(session.id.clone())),
        )
        .await;
        return;
    }
    match result {
        Ok(result) if result.has_agent_result => {
            super::fire_result::finish(
                occurrence_id,
                automation_id,
                &session.id,
                &request_id,
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
        Ok(_) => {
            super::fire_result::finish_error(
                occurrence_id,
                automation_id,
                &session.id,
                &request_id,
                "assistant_missing",
                Some(session.id.clone()),
            )
            .await
        }
        Err(error) if error == super::agentic::RUNTIME_ADMISSION_FAILED => {
            if session.created {
                delete_empty_session(&session.id).await;
            }
        }
        Err(error) => {
            if session.created {
                delete_empty_session(&session.id).await;
            }
            super::fire_result::finish_error(
                occurrence_id,
                automation_id,
                &session.id,
                &request_id,
                super::fire_result::error_code(&error),
                Some(session.id.clone()),
            )
            .await;
            let _ = app.emit(
                "wakeup-failed",
                serde_json::json!({"wakeup_id": automation_id, "error": "Le réveil a échoué"}),
            );
        }
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

#[cfg(test)]
pub(crate) async fn target_session_for_test(
    definition: &AutomationDefinition,
) -> Result<(String, bool), &'static str> {
    target_session(definition)
        .await
        .map(|target| (target.id, target.created))
}

async fn create_session(
    definition: &AutomationDefinition,
    project_id: Option<String>,
) -> Result<String, String> {
    let name = format!("⏰ {} • {}", definition.name, definition.model);
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

pub(super) async fn delete_empty_session(session_id: &str) {
    let Ok(session) = session_store::get(session_id).await else {
        return;
    };
    if session.messages.is_empty() && session_store::delete_one(session_id).await.is_err() {
        ::log::warn!("empty_automation_cleanup_failed");
    }
}

#[cfg(test)]
#[path = "fire_tests.rs"]
mod tests;
