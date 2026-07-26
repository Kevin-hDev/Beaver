use crate::services::agent_local::session_store;
use crate::services::agent_local::types_session::AgentSession;
use crate::services::agent_local::types_tools::ToolResult;

pub(super) enum DelegatePromptPersistence {
    AlreadyDelivered(String),
    Queued,
}

pub(super) fn has_coder_workspace(parent: &AgentSession) -> bool {
    parent.project_id.is_some()
        || !parent.working_dir.is_empty() && std::path::Path::new(&parent.working_dir).is_dir()
}

impl DelegatePromptPersistence {
    pub(super) fn initial_prompt(&self) -> Option<&str> {
        match self {
            Self::AlreadyDelivered(prompt) => Some(prompt),
            Self::Queued => None,
        }
    }
}

fn user_message(content: &str) -> super::types_session::AgentMessage {
    super::types_session::AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: content.to_string(),
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

pub(super) async fn persist_delegate_prompt(
    child_id: &str,
    prompt: &str,
    is_redeployment: bool,
) -> Result<DelegatePromptPersistence, ToolResult> {
    let result = if !is_redeployment {
        session_store::add_messages(child_id, vec![user_message(prompt)], 0)
            .await
            .map(|()| DelegatePromptPersistence::AlreadyDelivered(prompt.to_string()))
    } else {
        persist_redeployment_prompt(child_id, prompt).await
    };
    if let Ok(persisted) = result {
        return Ok(persisted);
    }
    eprintln!("[subagent] persistance prompt enfant {child_id}");
    let _ = super::session_subagents::mark_status(child_id, super::subagent_status::FAILED).await;
    Err(ToolResult::err(
        "Erreur interne lors de la création du sous-agent",
    ))
}

async fn persist_redeployment_prompt(
    child_id: &str,
    prompt: &str,
) -> Result<DelegatePromptPersistence, String> {
    super::session_store::validate_session_id(child_id)?;
    let lock = super::session_store::lock_session(child_id).await;
    let _guard = lock.lock().await;
    let mut child = super::session_store::get(child_id).await?;
    super::subagent_instruction_delivery::validate_persisted_queue(
        &child.subagent_queued_prompts,
    )?;
    if let Some(existing) = unanswered_matching_prompt(&child, prompt) {
        return Ok(DelegatePromptPersistence::AlreadyDelivered(existing));
    }
    super::subagent_instruction_delivery::enqueue(&mut child, prompt)
        .map_err(|result| result.content)?;
    super::session_store::save(&child).await?;
    Ok(DelegatePromptPersistence::Queued)
}

fn unanswered_matching_prompt(child: &AgentSession, prompt: &str) -> Option<String> {
    let last = child.messages.iter().rev().find(|message| message.role != "system")?;
    (last.role == "user"
        && super::subagent_instruction_delivery::normalized_prompt(&last.content)
            == super::subagent_instruction_delivery::normalized_prompt(prompt))
    .then(|| last.content.clone())
}

pub(super) async fn prepare_existing_child(
    child_id: &str,
    parent_session_id: &str,
    subagent_type: &str,
    prompt: &str,
    name: &str,
    description: &str,
    color_key: &str,
    run_id: &str,
) -> Result<AgentSession, ToolResult> {
    if session_store::validate_session_id(child_id).is_err() {
        return Err(ToolResult::err("Sous-agent introuvable."));
    }
    let lock = session_store::lock_session(child_id).await;
    let _guard = lock.lock().await;
    let mut child = match session_store::get(child_id).await {
        Ok(session) => session,
        Err(_) => return Err(ToolResult::err("Sous-agent introuvable.")),
    };
    if child.parent_session_id.as_deref() != Some(parent_session_id) {
        return Err(ToolResult::err("Sous-agent introuvable."));
    }
    if child.archived_at.is_some() {
        return Err(ToolResult::err("Sous-agent archivé."));
    }
    if super::subagent_live_state::has_pending_work(&child).await {
        return Err(ToolResult::err("Ce sous-agent est déjà en cours."));
    }
    child.name = name.to_string();
    child.subagent_type = Some(subagent_type.to_string());
    child.subagent_prompt = Some(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.subagent_run_id = Some(run_id.to_string());
    child.subagent_description = Some(description.to_string());
    child.subagent_color_key = Some(color_key.to_string());
    child.subagent_summary = None;
    session_store::save(&child)
        .await
        .map_err(|_| ToolResult::err("Erreur interne lors de la préparation du sous-agent"))?;
    Ok(child)
}

pub(super) async fn create_child(
    parent: &AgentSession,
    parent_session_id: &str,
    subagent_type: &str,
    prompt: &str,
    name: &str,
    description: &str,
    color_key: &str,
    run_id: &str,
) -> Result<AgentSession, ToolResult> {
    let mut child = session_store::create_full(
        name,
        &parent.model,
        &parent.provider,
        false,
        parent.project_id.clone(),
    )
    .await
    .map_err(|_| ToolResult::err("Erreur interne lors de la création du sous-agent"))?;
    child.parent_session_id = Some(parent_session_id.to_string());
    child.subagent_type = Some(subagent_type.to_string());
    child.subagent_prompt = Some(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.subagent_run_id = Some(run_id.to_string());
    child.subagent_description = Some(description.to_string());
    child.subagent_color_key = Some(color_key.to_string());
    child.thinking_enabled = parent.thinking_enabled;
    child.reasoning_mode = parent.reasoning_mode.clone();
    child.working_dir = parent.working_dir.clone();
    session_store::save(&child)
        .await
        .map_err(|_| ToolResult::err("Erreur interne lors de la création du sous-agent"))?;
    Ok(child)
}

pub(super) async fn inherit_parent_context(
    child: &mut AgentSession,
    parent: &AgentSession,
) -> Result<(), String> {
    child.model = parent.model.clone();
    child.provider = parent.provider.clone();
    child.thinking_enabled = parent.thinking_enabled;
    child.reasoning_mode = parent.reasoning_mode.clone();
    child.working_dir = parent.working_dir.clone();
    session_store::save(child).await
}
