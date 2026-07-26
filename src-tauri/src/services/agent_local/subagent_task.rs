use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_registry;
use crate::services::agent_local::types_session::AgentSession;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) use super::subagent_task_failure::finish_preparation_failure;

pub async fn run(
    app: AppHandle,
    parent_session_id: String,
    child_session_id: String,
    model: String,
    provider: String,
    runtime_context: super::subagent_runtime_context::SubagentRuntimeContext,
    prompt: String,
    subagent_type: String,
    parent_emitter: AgentEventEmitter,
    cancel: CancellationToken,
    project_id: Option<String>,
    run_id: String,
    execution_id: String,
) {
    if !subagent_registry::owns_execution(&child_session_id, &run_id, &execution_id).await {
        return;
    }
    let is_explorer = subagent_type == "explorer";
    let prepared = match super::subagent_working_dir::resolve(
        project_id.as_deref(),
        &child_session_id,
        is_explorer,
        &run_id,
        &execution_id,
    )
    .await
    {
        Ok(prepared) => prepared,
        Err(_) => {
            finish_preparation_failure(
                &parent_session_id,
                &child_session_id,
                &subagent_type,
                &run_id,
                &execution_id,
                Some(&parent_emitter),
            )
            .await;
            session_store::remove_session_lock(&child_session_id).await;
            return;
        }
    };
    let working_dir = prepared.path().to_string_lossy().to_string();
    let project_path = prepared.project_path().to_path_buf();
    let mut retain_branch = false;
    let mut prior_messages = None;
    loop {
        let Some(active) = subagent_registry::active_run_for_child(&child_session_id).await else {
            break;
        };
        if active.run_id != run_id || active.execution_id != execution_id {
            break;
        }
        if active.cancelled {
            let _ = super::subagent_completion_events::persist_terminal(
                &parent_session_id,
                &child_session_id,
                &subagent_type,
                super::subagent_status::CANCELLED,
                "Sous-agent annulé.",
                &run_id,
                &execution_id,
                false,
                Some(&parent_emitter),
            )
            .await;
            break;
        }
        let result = super::subagent_task_stream::run_inner(
            app.clone(),
            child_session_id.clone(),
            model.clone(),
            provider.clone(),
            runtime_context.clone(),
            parent_emitter.clone(),
            prompt.clone(),
            subagent_type.clone(),
            cancel.clone(),
            project_id.clone(),
            working_dir.clone(),
            prior_messages.take(),
        )
        .await;
        let (mut success, mut status, mut summary, completed_messages) = match result {
            Ok((success, status, summary, messages)) => {
                (success, status, summary, messages)
            }
            Err(error) if super::subagent_instruction_delivery::is_delivery_error(&error) => {
                let reported = super::subagent_completion_events::persist_instruction_failure(
                    &parent_session_id,
                    &child_session_id,
                    &subagent_type,
                    &run_id,
                    &execution_id,
                    Some(&parent_emitter),
                )
                .await
                .unwrap_or(false);
                if !reported {
                    break;
                }
                break;
            }
            Err(_) => {
                eprintln!("[subagent] échec {}", child_session_id);
                (
                    false,
                    super::subagent_status::FAILED.to_string(),
                    "Le sous-agent n'a pas pu terminer correctement.".to_string(),
                    None,
                )
            }
        };
        if let Some(messages) = completed_messages {
            match super::subagent_history::persist_for_execution(
                &child_session_id,
                &run_id,
                &execution_id,
                &messages,
            )
            .await
            {
                Ok(true) => prior_messages = Some(messages),
                Ok(false) => break,
                Err(_) => {
                    success = false;
                    status = super::subagent_status::FAILED.to_string();
                    summary = super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string();
                }
            }
        }
        if !is_explorer {
            match super::subagent_task_change::capture(
                &project_path,
                &child_session_id,
                &execution_id,
                prepared.path(),
            )
            .await
            {
                Ok(Some(metadata)) => {
                    summary.push_str(&metadata);
                    retain_branch = true;
                }
                Ok(None) => {}
                Err(_) => {
                    success = false;
                    status = super::subagent_status::FAILED.to_string();
                    summary = "Le changement du sous-agent n'a pas pu être conservé.".into();
                    retain_branch = true;
                }
            }
        }
        let finalized = match super::subagent_completion_events::persist_terminal(
            &parent_session_id,
            &child_session_id,
            &subagent_type,
            &status,
            &summary,
            &run_id,
            &execution_id,
            success,
            Some(&parent_emitter),
        )
        .await
        {
            Ok(Some(value)) => value,
            Ok(None) => break,
            Err(_) => break,
        };
        if finalized.queued_followup {
            continue;
        }
        break;
    }

    super::subagent_task_change::cleanup_execution(
        &project_path,
        &child_session_id,
        &execution_id,
        prepared.worktree_path(),
        is_explorer || retain_branch,
    ).await;
    session_store::remove_session_lock(&child_session_id).await;
}

include!("subagent_task_state.rs");
