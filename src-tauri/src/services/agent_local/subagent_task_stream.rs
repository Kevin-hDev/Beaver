#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::models::agent_turn_contract::NewUserTurnInput;
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::{self, AgentEventEmitter};
use crate::services::agent_local::types_ollama::StreamEvent;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) async fn run_inner(
    app: AppHandle,
    child_session_id: String,
    model: String,
    provider: String,
    runtime_context: super::subagent_runtime_context::SubagentRuntimeContext,
    permission_emitter: AgentEventEmitter,
    prompt: String,
    subagent_type: String,
    cancel: CancellationToken,
    _project_id: Option<String>,
    working_dir: String,
) -> Result<(bool, String, String), String> {
    let profile =
        super::subagent_tool_profile::SubagentToolProfile::from_session_type(Some(&subagent_type))?;
    let skills_enabled = super::agent_settings::is_tool_enabled("load_skill").await;
    let tools = profile.definitions(skills_enabled);
    let response_language = crate::commands::agent_chat_task::common::response_language();
    let system_prompt = super::subagent_prompts::system(
        profile,
        std::path::Path::new(&working_dir),
        skills_enabled,
        &response_language,
    )
    .await;

    let target = crate::commands::agent_chat_target::resolve(
        &child_session_id,
        &provider,
        &model,
        None,
        None,
    )
    .await
    .map_err(|_| "conversation_admission_failed".to_string())?;
    let active = super::subagent_registry::active_run_for_child(&child_session_id)
        .await
        .ok_or_else(|| "conversation_admission_failed".to_string())?;
    let admitted = admit_subagent_turn(
        &child_session_id,
        &prompt,
        target.continuation.clone(),
        &active.run_id,
        &active.execution_id,
    )
    .await?;
    let generation = stream_events::next_generation();
    let emitter = AgentEventEmitter::with_generation(app, child_session_id.clone(), generation);
    let request_id = super::stream_diagnostics::start_request(&child_session_id, generation).await;
    super::subagent_activity::record_status(&child_session_id, "Démarré", None).await;
    if let Ok(child_session) = session_store::get(&child_session_id).await {
        if let Ok(messages) = super::session_view::messages(&child_session.messages) {
            let _ = emitter.send(StreamEvent::SessionSnapshot {
                messages,
                token_count: child_session.accumulated_tokens,
            });
        }
    }

    let result = run_stream_task(StreamTaskParams {
        on_event: emitter.clone(),
        session_id: child_session_id.clone(),
        request_id: request_id.clone(),
        model,
        conversation: Some(
            crate::commands::agent_chat_task::StreamConversation::canonical_for_subagent(
                admitted,
                system_prompt,
                active.run_id,
                active.execution_id,
            ),
        ),
        continuation_target: Some(target.continuation),
        reasoning_profile: Some(target.reasoning.clone()),
        tools,
        think: target.reasoning.active,
        provider,
        working_dir: std::path::PathBuf::from(working_dir),
        outputs_dir: None,
        capability_hints: StreamCapabilityHints::default(),
        reasoning_mode: target.reasoning.mode_name,
        permission_mode: crate::commands::agent_chat_task::StreamPermissionMode::Bounded(Some(
            runtime_context.permission_mode,
        )),
        permission_emitter: Some(permission_emitter),
        parent_message_inbox: None,
        subagent_profile: Some(profile),
        plan_mode: Some(false),
        #[cfg(debug_assertions)]
        fixture_run: None,
        cancel: cancel.clone(),
    })
    .await;

    finalize_stream_result(
        result,
        &child_session_id,
        &request_id,
        cancel,
        Some(&emitter),
    )
    .await
}

async fn finalize_stream_result(
    result: Result<crate::services::agent_local::agent_loop_finish::CompletedStreamTurn, String>,
    child_session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    emitter: Option<&AgentEventEmitter>,
) -> Result<(bool, String, String), String> {
    let was_cancelled = cancel.is_cancelled();
    match result {
        Ok(completed) => {
            let summary =
                super::subagent_summary::extract_summary_from_messages(completed.messages());
            let status = if was_cancelled {
                super::subagent_status::CANCELLED
            } else {
                super::subagent_status::COMPLETED
            };
            if let Some(emitter) = emitter {
                completed.emit_done(emitter);
            }
            Ok((!was_cancelled, status.to_string(), summary))
        }
        Err(e) if was_cancelled || e == "Annulé" => Ok((
            false,
            super::subagent_status::CANCELLED.to_string(),
            "Sous-agent annulé.".to_string(),
        )),
        Err(e) if super::subagent_instruction_delivery::is_delivery_error(&e) => Err(e),
        Err(_) => {
            super::stream_diagnostics::record_failure(
                child_session_id,
                Some(request_id),
                "Le sous-agent n'a pas pu terminer correctement.",
                false,
            )
            .await;
            Err("Le sous-agent n'a pas pu terminer correctement.".to_string())
        }
    }
}

async fn admit_subagent_turn(
    session_id: &str,
    content: &str,
    target: crate::services::reasoning_continuity::contract::ContinuationTarget,
    run_id: &str,
    execution_id: &str,
) -> Result<crate::services::agent_local::conversation_admission::AdmittedTurn, String> {
    if !super::subagent_registry::owns_execution(session_id, run_id, execution_id).await {
        return Err("conversation_admission_failed".to_string());
    }
    let input = crate::services::agent_local::conversation_input::resolve(NewUserTurnInput {
        content: content.to_string(),
        files: Vec::new(),
        skills: Vec::new(),
    })
    .await
    .map_err(|_| "conversation_admission_failed".to_string())?;
    crate::services::agent_local::conversation_admission::new_turn_for_continuation(
        session_id, input, target,
    )
    .await
    .map_err(|_| "conversation_admission_failed".to_string())
}

#[cfg(test)]
#[path = "subagent_task_stream_tests.rs"]
mod tests;
