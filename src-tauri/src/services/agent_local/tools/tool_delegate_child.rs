#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::services::agent_local::session_store;
use crate::services::agent_local::types_session::AgentSession;
use crate::services::agent_local::types_tools::ToolResult;

pub(super) fn has_coder_workspace(parent: &AgentSession) -> bool {
    parent.project_id.is_some()
        || !parent.working_dir.is_empty() && std::path::Path::new(&parent.working_dir).is_dir()
}

pub(super) async fn persist_delegate_prompt(
    child_id: &str,
    prompt: &str,
) -> Result<(), ToolResult> {
    super::session_store::validate_session_id(child_id)
        .map_err(|_| ToolResult::validation("subagent_id_invalid", "Sous-agent introuvable."))?;
    if prompt.trim().is_empty()
        || prompt.chars().count() > super::subagent_instruction_delivery::MAX_PROMPT_SIZE
    {
        return Err(ToolResult::validation(
            "subagent_prompt_invalid",
            "Consigne sous-agent invalide.",
        ));
    }
    let child = session_store::get(child_id).await.map_err(|_| {
        ToolResult::internal(
            "subagent_prompt_save_failed",
            "Erreur interne lors de la création du sous-agent",
            false,
        )
    })?;
    super::subagent_instruction_delivery::validate_persisted_queue(&child.subagent_queued_prompts)
        .map_err(|_| {
            ToolResult::internal(
                "subagent_prompt_save_failed",
                "Erreur interne lors de la création du sous-agent",
                false,
            )
        })?;
    // L'admission canonique dans `subagent_task_stream::run_inner` est le
    // propriétaire unique de l'écriture du tour utilisateur.
    Ok(())
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
        return Err(ToolResult::validation(
            "subagent_id_invalid",
            "Sous-agent introuvable.",
        ));
    }
    let lock = session_store::lock_session(child_id).await;
    let _guard = lock.lock().await;
    let mut child = match session_store::get(child_id).await {
        Ok(session) => session,
        Err(_) => {
            return Err(ToolResult::not_found(
                "subagent_not_found",
                "Sous-agent introuvable.",
            ))
        }
    };
    if child.parent_session_id.as_deref() != Some(parent_session_id) {
        return Err(ToolResult::not_found(
            "subagent_not_found",
            "Sous-agent introuvable.",
        ));
    }
    if child.archived_at.is_some() {
        return Err(ToolResult::conflict(
            "subagent_archived",
            "Sous-agent archivé.",
        ));
    }
    if super::subagent_live_state::has_pending_work(&child).await {
        return Err(ToolResult::conflict(
            "subagent_already_running",
            "Ce sous-agent est déjà en cours.",
        ));
    }
    child.name = name.to_string();
    child.subagent_type = Some(subagent_type.to_string());
    child.subagent_prompt = Some(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.subagent_run_id = Some(run_id.to_string());
    child.subagent_description = Some(description.to_string());
    child.subagent_color_key = Some(color_key.to_string());
    child.subagent_summary = None;
    session_store::save(&child).await.map_err(|_| {
        ToolResult::internal(
            "subagent_prepare_save_failed",
            "Erreur interne lors de la préparation du sous-agent",
            false,
        )
        .with_error_hint("Inspecter le sous-agent avant de reprendre sa préparation.")
    })?;
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
    .map_err(|_| {
        ToolResult::internal(
            "subagent_create_failed",
            "Erreur interne lors de la création du sous-agent",
            false,
        )
    })?;
    child.parent_session_id = Some(parent_session_id.to_string());
    child.subagent_type = Some(subagent_type.to_string());
    child.subagent_prompt = Some(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.subagent_run_id = Some(run_id.to_string());
    child.subagent_description = Some(description.to_string());
    child.subagent_color_key = Some(color_key.to_string());
    child.thinking_enabled = parent.thinking_enabled;
    child.reasoning_mode = parent.reasoning_mode.clone();
    child.preserve_reasoning = parent.preserve_reasoning;
    child.compression_profile_selection = Some(
        crate::services::compress::profile_resolve::resolve_for_session(parent)
            .map_err(|_| {
                ToolResult::internal(
                    "subagent_compression_profile_failed",
                    "Erreur interne lors de la création du sous-agent",
                    false,
                )
            })?
            .selection(),
    );
    child.working_dir = parent.working_dir.clone();
    child.working_dir_managed = parent.working_dir_managed;
    session_store::save(&child).await.map_err(|_| {
        ToolResult::internal(
            "subagent_create_save_failed",
            "Erreur interne lors de la création du sous-agent",
            false,
        )
        .with_error_hint("Inspecter les sous-agents existants avant de relancer la création.")
    })?;
    Ok(child)
}

pub(super) async fn inherit_parent_context(
    child: &mut AgentSession,
    parent: &AgentSession,
) -> Result<(), String> {
    let lock = session_store::lock_session(&child.id).await;
    let _guard = lock.lock().await;
    let mut current = session_store::get(&child.id).await?;
    current.model = parent.model.clone();
    current.provider = parent.provider.clone();
    current.thinking_enabled = parent.thinking_enabled;
    current.reasoning_mode = parent.reasoning_mode.clone();
    current.preserve_reasoning = parent.preserve_reasoning;
    current.compression_profile_selection = Some(
        crate::services::compress::profile_resolve::resolve_for_session(parent)
            .map_err(|_| "Mise à jour du sous-agent impossible".to_string())?
            .selection(),
    );
    current.working_dir = parent.working_dir.clone();
    current.working_dir_managed = parent.working_dir_managed;
    session_store::save(&current).await?;
    *child = current;
    Ok(())
}
