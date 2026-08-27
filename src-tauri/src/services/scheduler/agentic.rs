use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::models::agent_turn_contract::{NewUserTurnInput, TurnStart};
use crate::models::ScheduledWakeup;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_ollama::StreamEvent;
#[cfg(test)]
use crate::services::agent_local::{conversation_admission, conversation_input};
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

pub struct ScheduledAgentResult {
    pub tokens: u32,
    pub has_text_result: bool,
}

pub async fn run(
    app: &AppHandle,
    wakeup: &ScheduledWakeup,
    session_id: &str,
    cancel: CancellationToken,
) -> Result<ScheduledAgentResult, String> {
    let target = crate::commands::agent_chat_target::resolve(
        session_id,
        &wakeup.provider,
        &wakeup.model,
        None,
        None,
    )
    .await?;
    let stream = crate::commands::agent_chat_admission::admit_background(app, session_id).await?;
    let streams = app.state::<crate::ActiveStreams>();
    let turn = crate::commands::agent_chat_turn::prepare(TurnStart::New(NewUserTurnInput {
        content: wakeup.prompt.clone(),
        files: Vec::new(),
        skills: Vec::new(),
    }))
    .await?;
    let admitted = match crate::commands::agent_chat_turn::admit_current(
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
    let admission_rollback = admitted.rollback();
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
    let emitter =
        AgentEventEmitter::with_generation(app.clone(), session_id.to_string(), stream.generation);
    let _ = emitter.send(StreamEvent::TurnAdmitted {
        turn_id: admitted.turn.turn_id.clone(),
        user_message_id: admitted.turn.user_message_id.clone(),
        assistant_message_id: admitted.turn.assistant_message_id.clone(),
    });
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
        model: wakeup.model.clone(),
        conversation: Some(
            crate::commands::agent_chat_task::StreamConversation::canonical(admitted.turn),
        ),
        continuation_target: Some(target.continuation),
        reasoning_profile: Some(target.reasoning.clone()),
        tools: Vec::new(),
        think: target.reasoning.active,
        provider: wakeup.provider.clone(),
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
    let has_text_result = completed
        .messages()
        .iter()
        .any(|message| message.role == "assistant" && !message.content.trim().is_empty());
    let tokens = generated_output_tokens(completed.messages());
    completed.emit_done(&emitter);
    Ok(ScheduledAgentResult {
        tokens,
        has_text_result,
    })
}

#[cfg(test)]
pub(crate) async fn admit_wakeup_turn(
    session_id: &str,
    prompt: &str,
    target: crate::services::reasoning_continuity::contract::ContinuationTarget,
) -> Result<conversation_admission::AdmittedTurn, String> {
    let input = conversation_input::resolve(NewUserTurnInput {
        content: prompt.to_string(),
        files: Vec::new(),
        skills: Vec::new(),
    })
    .await
    .map_err(|_| "conversation_admission_failed".to_string())?;
    conversation_admission::new_turn_for_continuation(session_id, input, target)
        .await
        .map_err(|_| "conversation_admission_failed".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

    #[test]
    fn token_estimate_includes_intermediate_answers_and_tool_calls() {
        let messages = vec![
            ChatMessage::assistant(
                "a".repeat(400),
                None,
                None,
                None,
                Some(vec![ToolCallOllama {
                    id: Some("call-1".into()),
                    extra_content: None,
                    function: ToolCallFunction {
                        name: "list_dir".into(),
                        arguments: serde_json::json!({"path": "."}),
                    },
                }]),
            ),
            ChatMessage::tool("README.md".into(), None, Some("list_dir".into())),
            ChatMessage::assistant("Terminé.".into(), None, None, None, None),
        ];

        assert!(generated_output_tokens(&messages) >= 100);
    }
}
