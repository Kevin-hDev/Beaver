use super::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};

use crate::services::agent_local::stream_events::{self, AgentEventEmitter};
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::ActiveStreams;
use std::sync::Arc;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn chat_stream(
    app: tauri::AppHandle,
    session_id: String,
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<serde_json::Value>,
    think: bool,
    provider: Option<String>,
    working_dir: Option<String>,
    supports_tools: Option<bool>,
    supports_thinking: Option<bool>,
    supports_vision: Option<bool>,
    reasoning_mode: Option<String>,
    permission_mode: Option<String>,
    plan_mode: Option<bool>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<u64, String> {
    let permission_mode = Some(
        crate::services::agent_local::session_permission_state::prepare_send(
            &session_id,
            permission_mode.as_deref(),
        )
        .await?,
    );
    let cancel = CancellationToken::new();
    let parent_message_inbox =
        Arc::new(crate::services::agent_local::parent_message_inbox::ParentMessageInbox::new());
    let generation = stream_events::next_generation();
    let cancelled_session_id = session_id.clone();
    let replacement_app = app.clone();
    let request_session_id = session_id.clone();
    let request_id = super::agent_chat_streams::replace_active_stream(
        &streams,
        &session_id,
        cancel.clone(),
        generation,
        parent_message_inbox.clone(),
        move |(old_token, _, old_request_id, old_inbox)| async move {
            crate::services::mascot::end_session(&replacement_app, &cancelled_session_id);
            old_inbox.close().await;
            crate::services::agent_local::session_locks::cancel_with_lock(
                &cancelled_session_id,
                &old_token,
            )
            .await;
            crate::services::agent_local::stream_diagnostics::record_cancelled(
                &cancelled_session_id,
                &old_request_id,
            )
            .await;
        },
        move || async move {
            crate::services::agent_local::stream_diagnostics::start_request(
                &request_session_id,
                generation,
            )
            .await
        },
    )
    .await?;
    let provider = provider.unwrap_or_else(|| "ollama".to_string());
    let resolved_working_dir =
        match super::agent_working_dir::resolve_for_session(&session_id, working_dir.as_deref())
            .await
        {
            Ok(dir) => dir,
            Err(err) => {
                streams.0.lock().await.remove(&session_id);
                crate::services::agent_local::stream_diagnostics::record_failure(
                    &session_id,
                    Some(&request_id),
                    &err,
                    false,
                )
                .await;
                return Err(err);
            }
        };
    let working_dir = Some(resolved_working_dir.path.to_string_lossy().to_string());
    crate::services::mascot::start_session(&app, &session_id);
    eprintln!("[stream] start session={session_id} gen={generation}");
    let stream_session = session_id.clone();
    let task_app = app.clone();

    tauri::async_runtime::spawn(async move {
        let emitter = AgentEventEmitter::with_generation(
            task_app.clone(),
            stream_session.clone(),
            generation,
        );
        let stream_request_id = request_id.clone();
        let result = run_stream_task(StreamTaskParams {
            on_event: emitter.clone(),
            session_id: stream_session.clone(),
            request_id: stream_request_id.clone(),
            model,
            messages,
            tools,
            think,
            provider,
            working_dir,
            capability_hints: StreamCapabilityHints {
                supports_tools,
                supports_thinking,
                supports_vision,
            },
            reasoning_mode,
            permission_mode_override: permission_mode,
            permission_emitter: None,
            parent_message_inbox: Some(parent_message_inbox.clone()),
            subagent_profile: None,
            plan_mode,
            cancel,
        })
        .await;
        parent_message_inbox.close().await;

        // Cleanup : ne supprime que si NOTRE génération est encore active
        // (une nouvelle requête a pu remplacer notre entrée)
        let is_current = {
            let state = task_app.state::<ActiveStreams>();
            let mut map = state.0.lock().await;
            match map.get(&stream_session) {
                Some((_, gen, _, _)) if *gen == generation => {
                    map.remove(&stream_session);
                    true
                }
                _ => false,
            }
        };

        if is_current {
            crate::services::agent_local::permission_gate::clear_session(&stream_session).await;
            crate::services::agent_local::session_store::remove_session_lock(&stream_session).await;
        }

        if let Err(message) = result {
            // Ne pas envoyer l'erreur "Annulé" — le frontend gère déjà le cancel
            // via stopSession(). Envoyer ce message tuerait un nouveau stream.
            if is_current && message != "Annulé" {
                let is_conn = message == "ollama_connection_lost";
                let diagnostic = crate::services::agent_local::stream_diagnostics::record_failure(
                    &stream_session,
                    Some(&stream_request_id),
                    &message,
                    is_conn,
                )
                .await;
                let _ = emitter.send(StreamEvent::Error {
                    message,
                    is_connection: is_conn,
                    diagnostic,
                });
            }
        }
    });

    Ok(generation)
}
