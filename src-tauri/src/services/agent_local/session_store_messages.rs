use super::types_session::{AgentMessage, AgentSession};

const MAX_MESSAGES_PER_SESSION: usize = 2_000;

pub(super) fn append_bounded(
    session: &mut AgentSession,
    new_messages: impl IntoIterator<Item = AgentMessage>,
) {
    session.messages.extend(new_messages);
    if session.messages.len() > MAX_MESSAGES_PER_SESSION {
        let excess = session.messages.len() - MAX_MESSAGES_PER_SESSION;
        session.messages.drain(..excess);
    }
}

pub(super) fn recompute_accumulated_tokens(session: &mut AgentSession) {
    session.accumulated_tokens =
        crate::services::token_counting::estimate_agent_messages_tokens(&session.messages);
}

pub async fn add_messages(
    id: &str,
    mut new_messages: Vec<AgentMessage>,
    tokens: u32,
) -> Result<(), String> {
    super::session_store::validate_session_id(id)?;
    for message in &new_messages {
        message.validate_stream_metadata()?;
    }
    let lock = super::session_store::lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(id).await?;
    let has_user_message = new_messages.iter().any(|message| message.role == "user");
    let todo_housekeeping =
        super::session_store_todos::apply_user_turn(&mut session, has_user_message);
    if tokens > 0 {
        if let Some(last) = new_messages.last_mut() {
            last.tokens = tokens;
        }
    }
    append_bounded(&mut session, new_messages);
    session.updated_at = Some(chrono::Utc::now());
    recompute_accumulated_tokens(&mut session);
    let result = super::session_store::save(&session).await;
    if result.is_ok() && todo_housekeeping.should_emit_empty_update {
        super::tool_todo::emit_update(id, Vec::new());
    }
    result
}

#[cfg(test)]
pub(super) async fn add_redeployment_prompt(id: &str, prompt: &str) -> Result<(), String> {
    add_redeployment_prompt_inner(id, prompt, || async {}).await
}

#[cfg(test)]
async fn add_redeployment_prompt_inner<F, Fut>(
    id: &str,
    prompt: &str,
    before_save: F,
) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    super::session_store::validate_session_id(id)?;
    if prompt.trim().is_empty()
        || prompt.chars().count() > super::subagent_instruction_delivery::MAX_PROMPT_SIZE
    {
        return Err("Instruction sous-agent invalide.".to_string());
    }
    let lock = super::session_store::lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(id).await?;
    super::subagent_instruction_delivery::validate_persisted_queue(
        &session.subagent_queued_prompts,
    )?;
    super::subagent_instruction_delivery::enqueue(&mut session, prompt)
        .map_err(|result| result.content)?;
    before_save().await;
    super::session_store::save(&session).await
}

#[cfg(test)]
pub(super) async fn add_redeployment_prompt_with_before_save<F, Fut>(
    id: &str,
    prompt: &str,
    before_save: F,
) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    add_redeployment_prompt_inner(id, prompt, before_save).await
}
