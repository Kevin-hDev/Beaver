use super::types_ollama::ChatMessage;
use super::types_session::{AgentMessage, ToolCallRequest, ToolCallRequestFunction};

const MAX_MESSAGES: usize = 2_000;

pub(super) async fn persist_for_execution(
    child_id: &str,
    run_id: &str,
    execution_id: &str,
    messages: &[ChatMessage],
) -> Result<bool, String> {
    persist_inner(child_id, run_id, execution_id, messages, || async {}).await
}

async fn persist_inner<F, Fut>(
    child_id: &str,
    run_id: &str,
    execution_id: &str,
    messages: &[ChatMessage],
    before_save: F,
) -> Result<bool, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    super::session_store::validate_session_id(child_id)?;
    let lock = super::session_store::lock_session(child_id).await;
    let _guard = lock.lock().await;
    let mut child = super::session_store::get(child_id).await?;
    if child.parent_session_id.is_none()
        || child.subagent_run_id.as_deref() != Some(run_id)
        || !super::subagent_registry::owns_execution(child_id, run_id, execution_id).await
    {
        return Ok(false);
    }
    child.messages = bounded_saved_messages(messages);
    super::session_store_messages::recompute_accumulated_tokens(&mut child);
    child.updated_at = Some(chrono::Utc::now());
    before_save().await;
    super::session_store::save(&child).await?;
    Ok(true)
}

fn bounded_saved_messages(messages: &[ChatMessage]) -> Vec<AgentMessage> {
    let mut saved = messages.iter().filter_map(to_saved).collect::<Vec<_>>();
    if saved.len() > MAX_MESSAGES {
        let mut first_kept = saved.len() - MAX_MESSAGES;
        while first_kept < saved.len() && saved[first_kept].role == "tool" {
            first_kept += 1;
        }
        saved.drain(..first_kept);
    }
    saved
}

fn to_saved(message: &ChatMessage) -> Option<AgentMessage> {
    if !matches!(message.role.as_str(), "user" | "assistant" | "tool") {
        return None;
    }
    let tool_calls = message.tool_calls.as_ref().map(|calls| {
        calls
            .iter()
            .map(|call| ToolCallRequest {
                extra_content: call.extra_content.clone(),
                function: ToolCallRequestFunction {
                    name: call.function.name.clone(),
                    arguments: call.function.arguments.clone(),
                },
            })
            .collect::<Vec<_>>()
    });
    if message.role != "tool"
        && message.content.trim().is_empty()
        && tool_calls.as_ref().is_none_or(Vec::is_empty)
        && message.reasoning_content.as_deref().is_none_or(str::is_empty)
    {
        return None;
    }
    Some(AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: message.role.clone(),
        content: message.content.clone(),
        thinking: message.reasoning_content.clone(),
        tool_calls,
        tool_name: message.tool_name.clone(),
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    })
}

#[cfg(test)]
pub(super) async fn persist_with_before_save<F, Fut>(
    child_id: &str,
    run_id: &str,
    execution_id: &str,
    messages: &[ChatMessage],
    before_save: F,
) -> Result<bool, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    persist_inner(child_id, run_id, execution_id, messages, before_save).await
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

    #[test]
    fn message_limit_never_keeps_a_tool_result_without_its_call() {
        let mut messages = vec![
            ChatMessage {
                role: "assistant".into(),
                content: "appel".into(),
                tool_calls: Some(vec![ToolCallOllama {
                    id: Some("call-boundary".into()),
                    extra_content: None,
                    function: ToolCallFunction {
                        name: "read_file".into(),
                        arguments: serde_json::json!({"path": "README.md"}),
                    },
                }]),
                ..Default::default()
            },
            ChatMessage {
                role: "tool".into(),
                content: "résultat".into(),
                tool_name: Some("read_file".into()),
                tool_call_id: Some("call-boundary".into()),
                ..Default::default()
            },
        ];
        messages.extend((0..1_999).map(|index| ChatMessage {
            role: "user".into(),
            content: format!("message-{index}"),
            ..Default::default()
        }));

        let saved = bounded_saved_messages(&messages);

        assert_eq!(saved.len(), 1_999);
        assert_eq!(saved[0].role, "user");
        assert!(saved.iter().all(|message| message.role != "tool"));
    }
}
