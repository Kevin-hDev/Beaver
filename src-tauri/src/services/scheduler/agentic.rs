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
    let reply = completed
        .iter()
        .rev()
        .find(|message| message.role == "assistant" && !message.content.trim().is_empty())
        .map(|message| message.content.clone())
        .ok_or_else(|| "L'automatisation n'a produit aucun résultat.".to_string())?;
    let tokens =
        crate::services::token_counting::estimate_text_tokens(&reply).min(u32::MAX as usize) as u32;
    Ok(ScheduledAgentResult {
        messages: completed,
        tokens,
    })
}

fn initial_messages(prompt: &str) -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: "user".into(),
        content: prompt.to_string(),
        ..Default::default()
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_prompt_leaves_system_context_to_the_agent_engine() {
        let messages = initial_messages("cherche les nouveautés");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "cherche les nouveautés");
    }
}
