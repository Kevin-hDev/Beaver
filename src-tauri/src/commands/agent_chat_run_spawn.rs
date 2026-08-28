#[allow(
    clippy::too_many_arguments,
    reason = "validated stream ownership is explicit"
)]
async fn spawn(
    app: tauri::AppHandle,
    request: ChatStreamRequest,
    streams: &ActiveStreams,
    stream: super::agent_chat_admission::AgentChatAdmission,
    work: super::agent_chat_work::AgentStreamAdmission,
    admitted: crate::services::agent_local::conversation_admission::AdmittedTurn,
    admission_rollback: super::agent_chat_turn::AdmissionRollback,
    target: super::agent_chat_target::ResolvedChatTarget,
    resolved_dir: super::agent_working_dir::ResolvedWorkingDir,
    result: ChatStreamAdmission,
) -> Result<(), String> {
    let session_id = request.session_id.clone();
    let task_session = session_id.clone();
    let task_app = app.clone();
    let task_cancel = stream.cancel.clone();
    let task_inbox = stream.parent_message_inbox.clone();
    let run_inbox = task_inbox.clone();
    let request_id = stream.request_id.clone();
    let permission_mode = stream.permission_mode.clone();
    let generation = stream.generation;
    let spawn_rollback = admission_rollback.clone();
    let emitter = AgentEventEmitter::with_generation(app.clone(), session_id.clone(), generation);
    let _ = emitter.send(StreamEvent::TurnAdmitted {
        turn_id: result.turn_id.clone(),
        user_message_id: result.user_message_id.clone(),
        assistant_message_id: result.assistant_message_id.clone(),
    });
    let spawn_result = super::agent_chat_work::spawn(
        work,
        stream.cancel.clone(),
        Box::pin(async move {
            let stream_request_id = request_id.clone();
            let outcome = run_stream_task(StreamTaskParams {
                on_event: emitter.clone(),
                session_id: task_session.clone(),
                request_id: stream_request_id.clone(),
                model: request.model,
                conversation: Some(StreamConversation::canonical(admitted)),
                continuation_target: Some(target.continuation),
                reasoning_profile: Some(target.reasoning),
                tools: request.tools,
                think: request.think,
                provider: request.provider,
                working_dir: resolved_dir.path,
                outputs_dir: resolved_dir.outputs_dir,
                capability_hints: request.capability_hints,
                reasoning_mode: request.reasoning_mode,
                permission_mode: super::agent_chat_task::StreamPermissionMode::Bounded(Some(
                    permission_mode,
                )),
                permission_emitter: None,
                parent_message_inbox: Some(run_inbox.clone()),
                subagent_profile: None,
                plan_mode: request.plan_mode,
                #[cfg(debug_assertions)]
                fixture_run: request.fixture_run,
                cancel: task_cancel,
            })
            .await;
            run_inbox.close().await;
            if outcome.is_err() {
                let streams = task_app.state::<ActiveStreams>();
                let _ = super::agent_chat_turn::rollback_current(
                    &streams,
                    &task_session,
                    generation,
                    &admission_rollback,
                )
                .await;
            }
            let is_current = cleanup_current(&task_app, &task_session, generation).await;
            match outcome {
                Ok(completed) if is_current => completed.emit_done(&emitter),
                Ok(_) => {}
                Err(message) => {
                    emit_failure(
                        is_current,
                        &emitter,
                        &task_session,
                        &stream_request_id,
                        message,
                    )
                    .await;
                }
            }
        }),
    );
    if let Err(error) = spawn_result {
        let _ = super::agent_chat_turn::rollback_current(
            streams,
            &session_id,
            generation,
            &spawn_rollback,
        )
        .await;
        rollback(streams, &session_id, &stream).await;
        return Err(error);
    }
    Ok(())
}

async fn cleanup_current(app: &tauri::AppHandle, session_id: &str, generation: u64) -> bool {
    let state = app.state::<ActiveStreams>();
    super::agent_chat_streams::finish_active_stream(&state, session_id, generation).await
}

async fn emit_failure(
    current: bool,
    emitter: &AgentEventEmitter,
    session_id: &str,
    request_id: &str,
    message: String,
) {
    if !current || message == "Annulé" {
        return;
    }
    let (public_message, context_capacity) =
        crate::services::agent_local::context_capacity_error::public_error(&message);
    let connection = crate::services::agent_local::stream_diagnostics_failure::is_connection_error(
        &public_message,
    );
    let diagnostic = crate::services::agent_local::stream_diagnostics::record_failure(
        session_id,
        Some(request_id),
        &public_message,
        connection,
    )
    .await;
    let _ = emitter.send(StreamEvent::Error {
        message: public_message,
        is_connection: connection,
        context_capacity,
        diagnostic,
    });
}
