use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::subagent_registry;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

const MAX_PROMPT_PREVIEW: usize = 120;

pub use super::tool_delegate_spawned::SpawnedSubagent;

pub async fn prepare_delegate(
    args: Value,
    app: AppHandle,
    parent_session_id: String,
    parent_emitter: AgentEventEmitter,
    parent_cancel: CancellationToken,
) -> Result<SpawnedSubagent, ToolResult> {
    let prompt = super::tool_delegate_prompt::from_args(&args)?;
    let subagent_type = match args["subagent_type"].as_str() {
        Some("explorer") => "explorer",
        Some("coder") => "coder",
        Some(_) => return Err(ToolResult::err("Type de sous-agent invalide.")),
        None => return Err(ToolResult::err("Paramètre 'subagent_type' manquant")),
    };
    let name = super::subagent_profile::clean_name(
        args["display_name"]
            .as_str()
            .or_else(|| args["name"].as_str()),
        subagent_type,
    );
    let legacy_label = super::subagent_profile::legacy_mission_label(
        args["display_name"]
            .as_str()
            .or_else(|| args["name"].as_str()),
        subagent_type,
    );
    let description_owned = args["description"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or(legacy_label);
    let description =
        super::subagent_profile::clean_description(description_owned.as_deref(), &prompt);
    let color_key = super::subagent_profile::default_color_key(subagent_type).to_string();

    let parent = match session_store::get(&parent_session_id).await {
        Ok(s) => s,
        Err(_) => {
            return Err(ToolResult::err(
                "Erreur interne lors de la création du sous-agent",
            ))
        }
    };
    if parent.parent_session_id.is_some() {
        return Err(ToolResult::err(
            "Les sous-agents ne peuvent pas lancer d'autres sous-agents.",
        ));
    }
    if subagent_type == "coder" && !super::tool_delegate_child::has_coder_workspace(&parent) {
        return Err(ToolResult::err(
            "Un sous-agent code doit être lancé depuis un dossier valide.",
        ));
    }

    let run_id = subagent_registry::get_or_create_run_id(&parent_session_id).await;

    let existing_child_id = args["subagent_id"]
        .as_str()
        .map(str::trim)
        .filter(|id| !id.is_empty());
    let mut child = match existing_child_id {
        Some(id) => {
            match super::tool_delegate_child::prepare_existing_child(
                id.trim(),
                &parent_session_id,
                subagent_type,
                &prompt,
                &name,
                &description,
                &color_key,
                &run_id,
            )
            .await
            {
                Ok(session) => session,
                Err(result) => {
                    subagent_registry::release_run_claim(&parent_session_id, &run_id).await;
                    return Err(result);
                }
            }
        }
        _ => {
            match super::tool_delegate_child::create_child(
                &parent,
                &parent_session_id,
                subagent_type,
                &prompt,
                &name,
                &description,
                &color_key,
                &run_id,
            )
            .await
            {
                Ok(session) => session,
                Err(result) => {
                    subagent_registry::release_run_claim(&parent_session_id, &run_id).await;
                    return Err(result);
                }
            }
        }
    };

    if super::tool_delegate_child::inherit_parent_context(&mut child, &parent)
        .await
        .is_err()
    {
        subagent_registry::release_run_claim(&parent_session_id, &run_id).await;
        return Err(ToolResult::err(
            "Erreur interne lors de la préparation du sous-agent",
        ));
    }

    let child_id = child.id.clone();

    let persisted_prompt = match super::tool_delegate_child::persist_delegate_prompt(
        &child_id,
        &prompt,
        existing_child_id.is_some(),
    )
    .await
    {
        Ok(persisted) => persisted,
        Err(result) => {
            subagent_registry::release_run_claim(&parent_session_id, &run_id).await;
            return Err(result);
        }
    };

    let cancel = CancellationToken::new();
    let initial_prompt = persisted_prompt.initial_prompt();
    let registered = match subagent_registry::register_execution_for_parent_stream(
        &parent_session_id,
        &child_id,
        cancel.clone(),
        initial_prompt,
        &parent_cancel,
    )
    .await
    {
        Ok(registered) => registered,
        Err(error) => {
            let _ = super::session_subagents::mark_status(
                &child_id,
                super::subagent_status::FAILED,
            )
            .await;
            subagent_registry::release_run_claim(&parent_session_id, &run_id).await;
            return Err(ToolResult::err(error));
        }
    };
    let run_id = registered.run_id;

    let prompt_preview: String = prompt.chars().take(MAX_PROMPT_PREVIEW).collect();
    let spawn_event = StreamEvent::SubagentSpawned {
        subagent_session_id: child_id.clone(),
        subagent_name: name.clone(),
        subagent_type: subagent_type.to_string(),
        subagent_description: description,
        subagent_color_key: color_key,
        prompt_preview: prompt_preview.clone(),
        run_id: Some(run_id.clone()),
    };

    Ok(SpawnedSubagent {
        app,
        child_id,
        model: parent.model.clone(),
        provider: parent.provider.clone(),
        runtime_context: super::subagent_runtime_context::SubagentRuntimeContext::from_parent(
            &parent,
        )
        .await,
        prompt,
        subagent_type: subagent_type.to_string(),
        parent_emitter,
        cancel,
        project_id: parent.project_id.clone(),
        run_id,
        execution_id: registered.execution_id,
        spawn_event,
    })
}
