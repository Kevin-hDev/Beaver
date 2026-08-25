use crate::models::{ScheduledWakeup, WakeupSchedule};
use crate::services::agent_local::session_store;
use crate::services::gateway::message_convert;
use crate::services::llm;
use crate::services::scheduler::log;
use chrono::{DateTime, Local};
use std::collections::VecDeque;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

pub(crate) use super::fire_once::{claim_once, run_wakeup_steps, WakeupStepOutcome};
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
    if llm::route::is_interactive_only(&wakeup.provider) {
        return Err("Provider réservé aux conversations manuelles".to_string());
    }
    let session_id = create_heartbeat_session(wakeup).await?;
    // Le scheduler ne possède pas de second moteur : tout réveil utilise le
    // contexte et les outils de l'Agent Local en accès complet.
    let result = super::agentic::run(app, wakeup, &session_id, cancel.clone()).await?;
    let tokens = persist_agent_result(&session_id, result).await?;
    Ok((session_id, tokens))
}

async fn persist_agent_result(
    session_id: &str,
    result: super::agentic::ScheduledAgentResult,
) -> Result<u32, String> {
    let mut messages = persisted_agent_messages(&result.messages);
    if let Some(message) = messages
        .iter_mut()
        .rev()
        .find(|message| message.role == "assistant")
    {
        message.tokens = result.tokens;
    }
    session_store::add_messages(session_id, messages, 0).await?;
    if !result.has_text_result {
        return Err("L'automatisation n'a produit aucun résultat.".to_string());
    }
    Ok(result.tokens)
}

fn persisted_agent_messages(
    completed: &[crate::services::agent_local::types_ollama::ChatMessage],
) -> Vec<crate::services::agent_local::types_session::AgentMessage> {
    let mut non_system = completed.iter().filter(|message| message.role != "system");
    non_system.next();
    let mut pending_calls = VecDeque::<(String, String)>::new();
    let mut persisted = Vec::new();
    for message in
        non_system.take(crate::services::agent_local::session_limits::MAX_MESSAGES_PER_SESSION)
    {
        let Some(mut saved) = message_convert::chat_to_agent_message(message) else {
            continue;
        };
        for call in saved.tool_calls.iter().flatten() {
            pending_calls.push_back((call.function.name.clone(), call.id.clone()));
        }
        if saved.role == "tool" {
            link_tool_result(&mut saved, &mut pending_calls);
        }
        persisted.push(saved);
    }
    persisted
}

fn link_tool_result(
    message: &mut crate::services::agent_local::types_session::AgentMessage,
    pending: &mut VecDeque<(String, String)>,
) {
    let position = message.tool_call_id.as_ref().map_or_else(
        || {
            pending.iter().position(|(name, _)| {
                message
                    .tool_name
                    .as_ref()
                    .is_none_or(|tool_name| tool_name == name)
            })
        },
        |id| pending.iter().position(|(_, pending_id)| pending_id == id),
    );
    if let Some((_, id)) = position.and_then(|position| pending.remove(position)) {
        message.tool_call_id.get_or_insert(id);
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
    if let Err(error) = session_store::add_messages(
        &session.id,
        vec![message_convert::new_user_agent_message(&wakeup.prompt)],
        0,
    )
    .await
    {
        let _ = session_store::delete_one(&session.id).await;
        return Err(error);
    }
    Ok(session.id)
}

#[cfg(test)]
#[path = "fire_tests.rs"]
mod tests;
