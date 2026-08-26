use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::models::agent_turn_contract::NewUserTurnInput;
use crate::models::ScheduledWakeup;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::{conversation_admission, conversation_input};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

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
    let admitted =
        admit_wakeup_turn(session_id, &wakeup.prompt, target.continuation.clone()).await?;
    // A projectless workspace needs the durable first user message as its label.
    let resolved_dir =
        crate::commands::agent_working_dir::resolve_for_session(session_id, None).await?;
    let emitter = AgentEventEmitter::new(app.clone(), session_id.to_string());
    let completed = run_stream_task(StreamTaskParams {
        on_event: emitter.clone(),
        session_id: session_id.to_string(),
        request_id: Uuid::new_v4().to_string(),
        model: wakeup.model.clone(),
        conversation: Some(
            crate::commands::agent_chat_task::StreamConversation::canonical(admitted),
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
        cancel,
    })
    .await?;
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
