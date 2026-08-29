use crate::models::{ScheduledWakeup, WakeupSchedule};
use crate::services::agent_local::session_store;
use crate::services::llm;
use crate::services::scheduler::log;
use chrono::{DateTime, Local};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

pub(crate) use super::fire_once::{claim_once, run_wakeup_steps, WakeupStepOutcome};
#[cfg(test)]
pub(crate) use super::fire_once::{claim_once_in, OnceClaimOutcome};

/// Déclenche un wakeup durable : l'admission du moteur enregistre le prompt avant
/// tout appel provider, puis le scheduler journalise l'exécution et l'événement frontend.
pub async fn fire_wakeup(
    app: AppHandle,
    wakeup: ScheduledWakeup,
    scheduled_for: DateTime<Local>,
    cancel: CancellationToken,
) {
    let result = run_wakeup_steps(
        matches!(wakeup.schedule, WakeupSchedule::Once { .. }),
        &cancel,
        || async { claim_once(&wakeup.id) },
        || dispatch(&app, &wakeup, &cancel),
    )
    .await;
    match result {
        Ok(WakeupStepOutcome::Completed((session_id, tokens))) => {
            warn_if_log_failed(log::log_ok(&wakeup.id, scheduled_for, &session_id, tokens).await);
            let _ = app.emit(
                "wakeup-completed",
                serde_json::json!({
                    "wakeup_id": wakeup.id,
                    "session_id": session_id,
                    "tokens": tokens,
                }),
            );
        }
        Ok(WakeupStepOutcome::SkippedInactive) => {}
        Ok(WakeupStepOutcome::Cancelled) => {
            warn_if_log_failed(log::log_cancelled(&wakeup.id, scheduled_for).await);
            ::log::info!("[scheduler] réveil ponctuel annulé pendant la fermeture");
        }
        Err(error) => {
            if cancel.is_cancelled() {
                return;
            }
            warn_if_log_failed(log::log_err(&wakeup.id, scheduled_for, &error).await);
            ::log::error!("[scheduler] échec d'un réveil");
            let _ = app.emit(
                "wakeup-failed",
                serde_json::json!({
                    "wakeup_id": wakeup.id,
                    "error": "Le réveil a échoué",
                }),
            );
        }
    }
}

fn warn_if_log_failed(result: Result<(), String>) {
    if result.is_err() {
        ::log::warn!("[scheduler] journal indisponible");
    }
}

async fn dispatch(
    app: &AppHandle,
    wakeup: &ScheduledWakeup,
    cancel: &CancellationToken,
) -> Result<(String, u32), String> {
    if !llm::stream_dispatch::is_available(
        &wakeup.provider,
        llm::stream_dispatch::InvocationKind::Interactive,
        llm::request_purpose::RequestPurpose::Automation,
    ) {
        return Err("Provider réservé aux conversations manuelles".to_string());
    }
    let session_id = create_heartbeat_session(wakeup).await?;
    // Le scheduler ne possède pas de second moteur : tout réveil utilise le
    // contexte et les outils de l'Agent Local en accès complet.
    let result = match super::agentic::run(app, wakeup, &session_id, cancel.clone()).await {
        Ok(result) => result,
        Err(error) => {
            delete_empty_heartbeat(&session_id).await;
            return Err(error);
        }
    };
    if !result.has_text_result {
        return Err("L'automatisation n'a produit aucun résultat.".to_string());
    }
    Ok((session_id, result.tokens))
}

async fn delete_empty_heartbeat(session_id: &str) {
    let Ok(session) = session_store::get(session_id).await else {
        return;
    };
    if session.messages.is_empty() && session_store::delete_one(session_id).await.is_err() {
        ::log::warn!("empty_heartbeat_cleanup_failed");
    }
}

async fn create_heartbeat_session(wakeup: &ScheduledWakeup) -> Result<String, String> {
    let name = if wakeup.provider == "ollama" {
        format!("Heartbeat • {} • {}", wakeup.name, wakeup.model)
    } else {
        format!(
            "Heartbeat • {} • {} • {}",
            wakeup.name, wakeup.provider, wakeup.model
        )
    };
    let session = session_store::create_with_project(
        &name,
        &wakeup.model,
        &wakeup.provider,
        true,
        wakeup.project_id.clone(),
    )
    .await?;
    Ok(session.id)
}

#[cfg(test)]
#[path = "fire_tests.rs"]
mod tests;
