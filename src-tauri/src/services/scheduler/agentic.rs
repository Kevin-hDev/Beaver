use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::models::ScheduledWakeup;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct ScheduledAgentResult {
    pub messages: Vec<ChatMessage>,
    pub tokens: u32,
    pub has_text_result: bool,
}

pub async fn run(
    app: &AppHandle,
    wakeup: &ScheduledWakeup,
    session_id: &str,
    cancel: CancellationToken,
) -> Result<ScheduledAgentResult, String> {
    let resolved_dir =
        crate::commands::agent_working_dir::resolve_for_session(session_id, None).await?;
    let messages = initial_messages(&wakeup.prompt);
    let completed = run_stream_task(StreamTaskParams {
        on_event: AgentEventEmitter::new(app.clone(), session_id.to_string()),
        session_id: session_id.to_string(),
        request_id: Uuid::new_v4().to_string(),
        model: wakeup.model.clone(),
        messages,
        tools: Vec::new(),
        think: false,
        provider: wakeup.provider.clone(),
        working_dir: resolved_dir.path,
        outputs_dir: resolved_dir.outputs_dir,
        capability_hints: StreamCapabilityHints::default(),
        reasoning_mode: None,
        permission_mode: crate::commands::agent_chat_task::StreamPermissionMode::FullAccess,
        permission_emitter: None,
        parent_message_inbox: None,
        subagent_profile: None,
        plan_mode: Some(false),
        cancel,
    })
    .await?;
    let has_text_result = completed
        .iter()
        .any(|message| message.role == "assistant" && !message.content.trim().is_empty());
    let tokens = generated_output_tokens(&completed);
    Ok(ScheduledAgentResult {
        messages: completed,
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

fn initial_messages(prompt: &str) -> Vec<ChatMessage> {
    vec![ChatMessage::user(prompt.to_string())]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

    #[test]
    fn scheduled_prompt_leaves_system_context_to_the_agent_engine() {
        let messages = initial_messages("cherche les nouveautés");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "cherche les nouveautés");
    }

    #[test]
    fn token_estimate_includes_intermediate_answers_and_tool_calls() {
        let messages = vec![
            ChatMessage::assistant(
                "a".repeat(400),
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
            ChatMessage::assistant("Terminé.".into(), None, None),
        ];

        assert!(generated_output_tokens(&messages) >= 100);
    }
}
