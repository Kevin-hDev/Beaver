use super::types_ollama::ChatMessage;
use super::types_session::{AgentMessage, AgentSession};
use super::types_tools::ToolResult;

pub(super) const DELIVERY_ERROR: &str = "Échec interne de livraison d'une consigne sous-agent.";
pub(super) const MAX_PROMPT_SIZE: usize = 50_000;
pub(super) const MAX_QUEUED_PROMPTS: usize = 8;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum EnqueueOutcome {
    Added,
    Duplicate,
}

pub(super) fn enqueue(
    child: &mut AgentSession,
    prompt: &str,
) -> Result<EnqueueOutcome, ToolResult> {
    if validate_persisted_queue(&child.subagent_queued_prompts).is_err()
        || prompt.trim().is_empty()
        || prompt.chars().count() > MAX_PROMPT_SIZE
    {
        return Err(ToolResult::err("File de consignes sous-agent invalide."));
    }
    let normalized = normalized_prompt(prompt);
    let already_queued = child
        .subagent_queued_prompts
        .iter()
        .any(|queued| normalized_prompt(queued) == normalized);
    if already_queued {
        return Ok(EnqueueOutcome::Duplicate);
    }
    if child.subagent_queued_prompts.len() >= MAX_QUEUED_PROMPTS {
        return Err(ToolResult::err("File de consignes sous-agent pleine."));
    }
    child.subagent_queued_prompts.push(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.updated_at = Some(chrono::Utc::now());
    Ok(EnqueueOutcome::Added)
}

pub(super) fn normalized_prompt(prompt: &str) -> String {
    prompt.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) async fn drain(
    session_id: &str,
    messages: &mut Vec<ChatMessage>,
) -> Result<usize, String> {
    drain_inner(session_id, messages, || async {}, || async {}).await
}

async fn drain_inner<FR, FRFut, FS, FSFut>(
    session_id: &str,
    messages: &mut Vec<ChatMessage>,
    after_registry_read: FR,
    before_save: FS,
) -> Result<usize, String>
where
    FR: FnOnce() -> FRFut,
    FRFut: std::future::Future<Output = ()>,
    FS: FnOnce() -> FSFut,
    FSFut: std::future::Future<Output = ()>,
{
    let Some(expected_run) = super::subagent_registry::active_run_for_child(session_id).await else {
        let session = super::session_store::get(session_id)
            .await
            .map_err(|_| delivery_error())?;
        return if session.parent_session_id.is_none()
            && session.subagent_run_id.is_none()
            && session.subagent_queued_prompts.is_empty()
        {
            Ok(0)
        } else {
            Err(delivery_error())
        };
    };
    if expected_run.cancelled {
        return Err(delivery_error());
    }
    after_registry_read().await;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut child = super::session_store::get(session_id)
        .await
        .map_err(|_| delivery_error())?;
    if child.parent_session_id.is_none() {
        return Err(delivery_error());
    }
    ensure_current_run(&child, &expected_run).await?;
    validate_persisted_queue(&child.subagent_queued_prompts)?;
    if child.subagent_queued_prompts.is_empty() {
        return Ok(0);
    }
    let prompts = child.subagent_queued_prompts.clone();
    super::session_store_messages::append_bounded(
        &mut child,
        prompts.iter().map(|prompt| agent_message(prompt)),
    );
    super::session_store_messages::recompute_accumulated_tokens(&mut child);
    child.subagent_queued_prompts.clear();
    child.updated_at = Some(chrono::Utc::now());
    before_save().await;
    super::subagent_registry::save_and_mark_prompts_delivered(
        &child,
        &expected_run.execution_id,
        &prompts,
    )
    .await
    .map_err(|_| delivery_error())?;
    messages.extend(prompts.iter().map(|prompt| ChatMessage {
        role: "user".to_string(),
        content: prompt.clone(),
        ..Default::default()
    }));
    Ok(prompts.len())
}

pub(super) fn validate_persisted_queue(prompts: &[String]) -> Result<(), String> {
    if prompts.len() > MAX_QUEUED_PROMPTS {
        return Err(delivery_error());
    }
    for (index, prompt) in prompts.iter().enumerate() {
        if prompt.trim().is_empty() || prompt.chars().count() > MAX_PROMPT_SIZE {
            return Err(delivery_error());
        }
        let normalized = normalized_prompt(prompt);
        if prompts[..index]
            .iter()
            .any(|seen| normalized_prompt(seen) == normalized)
        {
            return Err(delivery_error());
        }
    }
    Ok(())
}

#[cfg(test)]
include!("subagent_instruction_delivery_test_hooks.rs");

async fn ensure_current_run(
    child: &AgentSession,
    expected_run: &super::subagent_registry::ActiveSubagentRun,
) -> Result<(), String> {
    let active_run = super::subagent_registry::active_run_for_child(&child.id)
        .await
        .ok_or_else(delivery_error)?;
    if active_run.cancelled
        || active_run.run_id != expected_run.run_id
        || active_run.execution_id != expected_run.execution_id
        || child.subagent_run_id.as_deref() != Some(&expected_run.run_id)
    {
        return Err(delivery_error());
    }
    Ok(())
}

pub(super) fn agent_message(prompt: &str) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: prompt.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}

fn delivery_error() -> String {
    DELIVERY_ERROR.to_string()
}

pub(super) fn is_delivery_error(error: &str) -> bool {
    error == DELIVERY_ERROR
}
