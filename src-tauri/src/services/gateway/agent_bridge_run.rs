use super::agent_bridge::BridgeError;
use super::agent_bridge_support::{audit_msg, emit_session_updated, send_final_reply};
use super::channels::{ChannelAdapter, InboundMessage};
use super::security::audit::{self, AuditAction};
use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::models::agent_turn_contract::{NewUserTurnInput, TurnStart};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

pub(super) async fn run(
    app: tauri::AppHandle,
    msg: &InboundMessage,
    adapter: &dyn ChannelAdapter,
    cancel: CancellationToken,
    session_id: String,
    provider: String,
    model: String,
) -> Result<(), BridgeError> {
    let target =
        crate::commands::agent_chat_target::resolve(&session_id, &provider, &model, None, None)
            .await
            .map_err(|_| BridgeError::SessionError("conversation_admission_failed".into()))?;
    let stream = crate::commands::agent_chat_admission::admit_background(&app, &session_id)
        .await
        .map_err(|error| BridgeError::SessionError(audit::sanitize_error(&error)))?;
    let turn = crate::commands::agent_chat_turn::prepare(TurnStart::New(NewUserTurnInput {
        content: msg.content.clone(),
        files: Vec::new(),
        skills: Vec::new(),
    }))
    .await
    .map_err(BridgeError::SessionError)?;
    let streams = app.state::<crate::ActiveStreams>();
    let admitted = match crate::commands::agent_chat_turn::admit_current(
        &streams,
        &session_id,
        stream.generation,
        turn,
        target.continuation.clone(),
        target.session_reasoning.clone(),
    )
    .await
    {
        Ok(admitted) => admitted,
        Err(error) => {
            crate::commands::agent_chat_streams::finish_active_stream(
                &streams,
                &session_id,
                stream.generation,
            )
            .await;
            return Err(BridgeError::SessionError(error));
        }
    };
    let admission_rollback = admitted.rollback();
    emit_session_updated(&app, &session_id);
    let resolved_working_dir =
        match crate::commands::agent_working_dir::resolve_for_session(&session_id, None).await {
            Ok(directory) => directory,
            Err(error) => {
                let _ = crate::commands::agent_chat_turn::rollback_current(
                    &streams,
                    &session_id,
                    stream.generation,
                    &admission_rollback,
                )
                .await;
                crate::commands::agent_chat_streams::finish_active_stream(
                    &streams,
                    &session_id,
                    stream.generation,
                )
                .await;
                return Err(BridgeError::SessionError(error));
            }
        };
    let emitter =
        AgentEventEmitter::with_generation(app.clone(), session_id.clone(), stream.generation);
    let _ = emitter.send(StreamEvent::TurnAdmitted {
        turn_id: admitted.turn.turn_id.clone(),
        user_message_id: admitted.turn.user_message_id.clone(),
        assistant_message_id: admitted.turn.assistant_message_id.clone(),
    });
    let request_id = stream.request_id.clone();
    let run_cancel = stream.cancel.clone();
    let linked_cancel = run_cancel.clone();
    let cancel_link = tokio::spawn(async move {
        cancel.cancelled().await;
        linked_cancel.cancel();
    });
    let completed = match run_stream_task(StreamTaskParams {
        on_event: emitter.clone(),
        session_id: session_id.clone(),
        request_id: request_id.clone(),
        model,
        conversation: Some(
            crate::commands::agent_chat_task::StreamConversation::canonical(admitted.turn),
        ),
        continuation_target: Some(target.continuation),
        reasoning_profile: Some(target.reasoning.clone()),
        tools: vec![],
        think: target.reasoning.active,
        provider,
        working_dir: resolved_working_dir.path,
        outputs_dir: resolved_working_dir.outputs_dir,
        capability_hints: StreamCapabilityHints::default(),
        reasoning_mode: target.reasoning.mode_name,
        permission_mode: crate::commands::agent_chat_task::StreamPermissionMode::Bounded(Some(
            "auto".to_string(),
        )),
        permission_emitter: None,
        parent_message_inbox: None,
        subagent_profile: None,
        plan_mode: Some(false),
        #[cfg(debug_assertions)]
        fixture_run: None,
        cancel: run_cancel,
    })
    .await
    {
        Ok(completed) => completed,
        Err(error) => {
            cancel_link.abort();
            let _ = crate::commands::agent_chat_turn::rollback_current(
                &streams,
                &session_id,
                stream.generation,
                &admission_rollback,
            )
            .await;
            let current = crate::commands::agent_chat_streams::finish_active_stream(
                &streams,
                &session_id,
                stream.generation,
            )
            .await;
            if !current {
                return Err(BridgeError::AgentError(
                    crate::commands::agent_chat_streams::STREAM_REPLACED.to_string(),
                ));
            }
            crate::services::agent_local::stream_diagnostics::record_failure(
                &session_id,
                Some(&request_id),
                &error,
                false,
            )
            .await;
            let safe = audit::sanitize_error(&error);
            audit_msg(msg, AuditAction::AgentError, None, Some(&safe))?;
            return Err(BridgeError::AgentError(safe));
        }
    };
    cancel_link.abort();
    if !crate::commands::agent_chat_streams::finish_active_stream(
        &streams,
        &session_id,
        stream.generation,
    )
    .await
    {
        return Err(BridgeError::AgentError(
            crate::commands::agent_chat_streams::STREAM_REPLACED.to_string(),
        ));
    }
    emit_session_updated(&app, &session_id);
    send_final_reply(msg, adapter, completed.messages()).await?;
    completed.emit_done(&emitter);
    Ok(())
}
