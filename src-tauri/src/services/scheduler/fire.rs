use crate::models::{ScheduledWakeup, WakeupSchedule};
use crate::services::agent_local::ollama_stream;
use crate::services::agent_local::session_store;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentMessage;
use crate::services::llm;
use crate::services::scheduler::log;
use chrono::{DateTime, Local, Utc};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub(crate) use super::fire_once::{
    claim_once, missed_once_action, run_wakeup_steps, MissedOnceAction, WakeupStepOutcome,
};
#[cfg(test)]
pub(crate) use super::fire_once::{claim_once_in, OnceClaimOutcome};

/// Déclenche un wakeup : trouve/crée la conversation Heartbeat pour le modèle,
/// envoie le prompt à Ollama, append les messages, log l'exécution et émet
/// l'événement frontend. Un réveil ponctuel est revendiqué avant tout appel provider.
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
            log::log_ok(&wakeup.id, scheduled_for, &session_id, tokens).await;
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
            log::log_cancelled(&wakeup.id, scheduled_for).await;
            ::log::info!("[scheduler] réveil ponctuel annulé pendant la fermeture");
        }
        Err(error) => {
            if cancel.is_cancelled() {
                return;
            }
            log::log_err(&wakeup.id, scheduled_for, &error).await;
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

async fn dispatch(
    app: &AppHandle,
    wakeup: &ScheduledWakeup,
    cancel: &CancellationToken,
) -> Result<(String, u32), String> {
    if llm::route::is_interactive_only(&wakeup.provider) {
        return Err("Provider réservé aux conversations manuelles".to_string());
    }
    let session_id = find_or_create_heartbeat_session(&wakeup.provider, &wakeup.model).await?;
    // Route selon provider : Ollama (local) ou LLM API (via catalog).
    let (reply, tokens) = if wakeup.agentic {
        super::agentic::run(app, wakeup, &session_id, cancel.clone()).await?
    } else {
        let response = async {
            if wakeup.provider == "ollama" {
                ollama_stream::collect_chat(
                    &wakeup.model,
                    vec![ChatMessage {
                        role: "user".into(),
                        content: wakeup.prompt.clone(),
                        images: None,
                        tool_calls: None,
                        tool_name: None,
                        tool_call_id: None,
                        reasoning_content: None,
                    }],
                )
                .await
            } else {
                llm::collect_chat(
                    &wakeup.provider,
                    &wakeup.model,
                    &wakeup.prompt,
                    Some(&session_id),
                )
                .await
            }
        };
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err("cancelled".to_string()),
            result = response => result?,
        }
    };

    let user_msg = AgentMessage {
        id: Uuid::new_v4().to_string(),
        role: "user".into(),
        content: wakeup.prompt.clone(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    };

    let assistant_msg = AgentMessage {
        id: Uuid::new_v4().to_string(),
        role: "assistant".into(),
        content: reply,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: Utc::now(),
        tokens,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    };

    session_store::add_messages(&session_id, vec![user_msg, assistant_msg], tokens).await?;
    Ok((session_id, tokens))
}

async fn find_or_create_heartbeat_session(provider: &str, model: &str) -> Result<String, String> {
    if let Some(id) = session_store::find_heartbeat_session(provider, model).await? {
        return Ok(id);
    }
    let name = if provider == "ollama" {
        format!("Heartbeat • {}", model)
    } else {
        format!("Heartbeat • {} • {}", provider, model)
    };
    let session = session_store::create_with_flags(&name, model, provider, true).await?;
    // La conserver même après un échec évite de supprimer une session déjà
    // utilisée par un autre réveil concurrent et stabilise la clé de cache.
    Ok(session.id)
}
