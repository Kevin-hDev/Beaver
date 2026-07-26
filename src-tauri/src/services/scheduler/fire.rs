use crate::models::{ScheduledWakeup, WakeupSchedule};
use crate::services::agent_local::ollama_stream;
use crate::services::agent_local::session_store;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentMessage;
use crate::services::config as cfg;
use crate::services::llm;
use crate::services::scheduler::log;
use chrono::{DateTime, Local, Utc};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

/// Déclenche un wakeup : trouve/crée la conversation Heartbeat pour le modèle,
/// envoie le prompt à Ollama, append les messages, log l'exécution et émet
/// l'événement frontend. Si Once → marque active=false.
pub async fn fire_wakeup(app: AppHandle, wakeup: ScheduledWakeup, scheduled_for: DateTime<Local>) {
    match dispatch(&app, &wakeup).await {
        Ok((session_id, tokens)) => {
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
        Err(e) => {
            log::log_err(&wakeup.id, scheduled_for, &e).await;
            eprintln!("[scheduler] fire_wakeup {} error", wakeup.id);
            let _ = app.emit(
                "wakeup-failed",
                serde_json::json!({
                    "wakeup_id": wakeup.id,
                    "error": "Le réveil a échoué",
                }),
            );
        }
    }

    // Marquage Once → active=false, quel que soit le résultat
    if matches!(wakeup.schedule, WakeupSchedule::Once { .. }) {
        if let Err(e) = deactivate_once(&wakeup.id) {
            eprintln!("[scheduler] deactivate once {}: {}", wakeup.id, e);
        }
    }
}

async fn dispatch(_app: &AppHandle, wakeup: &ScheduledWakeup) -> Result<(String, u32), String> {
    if llm::route::is_interactive_only(&wakeup.provider) {
        return Err("Provider réservé aux conversations manuelles".to_string());
    }
    // 1. Appel LLM EN PREMIER : si fail, on ne crée aucune session vide.
    //    Route selon provider : Ollama (local) ou LLM API (via catalog).
    let (reply, tokens) = if wakeup.provider == "ollama" {
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
        .await?
    } else {
        llm::collect_chat(&wakeup.provider, &wakeup.model, &wakeup.prompt).await?
    };

    // 2. Ollama a répondu → on peut créer/trouver la session et append les messages.
    let session_id = find_or_create_heartbeat_session(&wakeup.provider, &wakeup.model).await?;

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
    Ok(session.id)
}

pub(crate) fn deactivate_once(id: &str) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    if let Some(w) = config.scheduled_wakeups.iter_mut().find(|w| w.id == id) {
        w.active = false;
        cfg::write_config(&config)?;
    }
    Ok(())
}
