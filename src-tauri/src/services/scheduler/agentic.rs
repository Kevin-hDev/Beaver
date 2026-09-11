use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::models::agent_turn_contract::{NewUserTurnInput, TurnStart};
use crate::models::AutomationDefinition;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub(super) const RUNTIME_ADMISSION_FAILED: &str = "automation_runtime_admission_failed";

pub struct ScheduledAgentResult {
    pub tokens: u32,
    pub has_text_result: bool,
}

pub async fn run(
    app: &AppHandle,
    automation: &AutomationDefinition,
    occurrence_id: Uuid,
    session_id: &str,
    stream: crate::commands::agent_chat_admission::AgentChatAdmission,
    cancel: CancellationToken,
) -> Result<ScheduledAgentResult, String> {
    let streams = app.state::<crate::ActiveStreams>();
    let target = match crate::commands::agent_chat_target::resolve(
        session_id,
        &automation.provider,
        &automation.model,
        None,
        None,
    )
    .await
    {
        Ok(target) => target,
        Err(error) => {
            crate::commands::agent_chat_streams::finish_active_stream(
                &streams,
                session_id,
                stream.generation,
            )
            .await;
            return Err(error.ui_code().to_string());
        }
    };
    let turn = crate::commands::agent_chat_turn::prepare(TurnStart::New(NewUserTurnInput {
        content: automation.prompt.clone(),
        files: Vec::new(),
        skills: Vec::new(),
    }))
    .await?;
    let admitted = match crate::commands::agent_chat_turn::admit_automation_current(
        &streams,
        session_id,
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
                session_id,
                stream.generation,
            )
            .await;
            return Err(error);
        }
    };
    let mut admission_rollback = admitted.rollback();
    // A projectless workspace needs the durable first user message as its label.
    let resolved_dir =
        match crate::commands::agent_working_dir::resolve_for_session(session_id, None).await {
            Ok(directory) => directory,
            Err(error) => {
                let _ = crate::commands::agent_chat_turn::rollback_current(
                    &streams,
                    session_id,
                    stream.generation,
                    &admission_rollback,
                )
                .await;
                crate::commands::agent_chat_streams::finish_active_stream(
                    &streams,
                    session_id,
                    stream.generation,
                )
                .await;
                return Err(error);
            }
        };
    if super::runtime::mark_running(occurrence_id, chrono::Utc::now())
        .await
        .is_err()
    {
        let _ = crate::commands::agent_chat_turn::rollback_current(
            &streams,
            session_id,
            stream.generation,
            &admission_rollback,
        )
        .await;
        crate::commands::agent_chat_streams::finish_active_stream(
            &streams,
            session_id,
            stream.generation,
        )
        .await;
        return Err(RUNTIME_ADMISSION_FAILED.to_string());
    }
    let emitter =
        AgentEventEmitter::with_generation(app.clone(), session_id.to_string(), stream.generation);
    let _ = emitter.send(admission_rollback.accept_execution_event());
    let run_cancel = stream.cancel.clone();
    let linked_cancel = run_cancel.clone();
    let cancel_link = tokio::spawn(async move {
        cancel.cancelled().await;
        linked_cancel.cancel();
    });
    let outcome = run_stream_task(StreamTaskParams {
        on_event: emitter.clone(),
        session_id: session_id.to_string(),
        request_id: stream.request_id.clone(),
        model: automation.model.clone(),
        conversation: Some(
            crate::commands::agent_chat_task::StreamConversation::canonical_for_automation(
                admitted.turn,
                automation.id,
                &automation.target,
            ),
        ),
        continuation_target: Some(target.continuation),
        reasoning_profile: Some(target.reasoning.clone()),
        tools: Vec::new(),
        think: target.reasoning.active,
        provider: automation.provider.clone(),
        working_dir: resolved_dir.path,
        outputs_dir: resolved_dir.outputs_dir,
        capability_hints: StreamCapabilityHints::default(),
        reasoning_mode: target.reasoning.mode_name,
        permission_mode: crate::commands::agent_chat_task::StreamPermissionMode::FullAccess,
        permission_emitter: None,
        parent_message_inbox: None,
        subagent_profile: None,
        plan_mode: Some(false),
        #[cfg(debug_assertions)]
        fixture_run: None,
        cancel: run_cancel,
    })
    .await;
    cancel_link.abort();
    let completed = match outcome {
        Ok(completed) => completed,
        Err(error) => {
            let _ = crate::commands::agent_chat_turn::rollback_current(
                &streams,
                session_id,
                stream.generation,
                &admission_rollback,
            )
            .await;
            crate::commands::agent_chat_streams::finish_active_stream(
                &streams,
                session_id,
                stream.generation,
            )
            .await;
            return Err(error);
        }
    };
    if !crate::commands::agent_chat_streams::finish_active_stream(
        &streams,
        session_id,
        stream.generation,
    )
    .await
    {
        return Err(crate::commands::agent_chat_streams::STREAM_REPLACED.to_string());
    }
    let has_text_result = has_text_result(completed.messages());
    let tokens = generated_output_tokens(completed.messages());
    completed.emit_done(&emitter);
    Ok(ScheduledAgentResult {
        tokens,
        has_text_result,
    })
}

fn generated_output_tokens(messages: &[ChatMessage]) -> u32 {
    messages
        .iter()
        .filter(|message| message.role == "assistant")
        .fold(0usize, |total, message| {
            total.saturating_add(
                crate::services::token_counting::estimate_chat_message_tokens(message),
            )
        })
        .min(u32::MAX as usize) as u32
}

pub(super) fn has_text_result(messages: &[ChatMessage]) -> bool {
    messages
        .iter()
        .any(|message| message.role == "assistant" && !message.content.trim().is_empty())
}

#[cfg(test)]
#[path = "agentic_tests.rs"]
mod tests;
