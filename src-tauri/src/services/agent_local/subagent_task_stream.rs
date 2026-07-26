use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::{self, AgentEventEmitter};
use crate::services::agent_local::types_ollama::StreamEvent;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) async fn run_inner(
    app: AppHandle,
    child_session_id: String,
    model: String,
    provider: String,
    runtime_context: super::subagent_runtime_context::SubagentRuntimeContext,
    permission_emitter: AgentEventEmitter,
    prompt: String,
    subagent_type: String,
    cancel: CancellationToken,
    _project_id: Option<String>,
    working_dir: String,
    prior_messages: Option<Vec<super::types_ollama::ChatMessage>>,
) -> Result<
    (
        bool,
        String,
        String,
        Option<Vec<super::types_ollama::ChatMessage>>,
    ),
    String,
> {
    let profile = super::subagent_tool_profile::SubagentToolProfile::from_session_type(Some(
        &subagent_type,
    ))?;
    let skills_enabled = super::agent_settings::is_tool_enabled("load_skill").await;
    let tools = profile.definitions(skills_enabled);
    let response_language = crate::commands::agent_chat_task::common::response_language();
    let system_prompt = super::subagent_prompts::system(
        profile,
        std::path::Path::new(&working_dir),
        skills_enabled,
        &response_language,
    )
    .await;

    let messages = super::subagent_context::build_messages(
        &child_session_id,
        system_prompt,
        &prompt,
        prior_messages,
    )
    .await;
    let generation = stream_events::next_generation();
    let emitter =
        AgentEventEmitter::with_generation(app, child_session_id.clone(), generation);
    let request_id =
        super::stream_diagnostics::start_request(&child_session_id, generation).await;
    super::subagent_activity::record_status(&child_session_id, "Démarré", None).await;
    if let Ok(child_session) = session_store::get(&child_session_id).await {
        let _ = emitter.send(StreamEvent::SessionSnapshot {
            messages: child_session.messages,
            token_count: child_session.accumulated_tokens,
        });
    }

    let result = run_stream_task(StreamTaskParams {
        on_event: emitter,
        session_id: child_session_id.clone(),
        request_id: request_id.clone(),
        model,
        messages,
        tools,
        think: runtime_context.think,
        provider,
        working_dir: Some(working_dir),
        capability_hints: StreamCapabilityHints::default(),
        reasoning_mode: runtime_context.reasoning_mode,
        permission_mode_override: Some(runtime_context.permission_mode),
        permission_emitter: Some(permission_emitter),
        parent_message_inbox: None,
        subagent_profile: Some(profile),
        plan_mode: Some(false),
        cancel: cancel.clone(),
    })
    .await;

    finalize_stream_result(result, &child_session_id, &request_id, cancel).await
}

async fn finalize_stream_result(
    result: Result<Vec<super::types_ollama::ChatMessage>, String>,
    child_session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
) -> Result<
    (
        bool,
        String,
        String,
        Option<Vec<super::types_ollama::ChatMessage>>,
    ),
    String,
> {
    let was_cancelled = cancel.is_cancelled();
    match result {
        Ok(final_msgs) => {
            let summary = super::subagent_summary::extract_summary_from_messages(&final_msgs);
            let status = if was_cancelled {
                super::subagent_status::CANCELLED
            } else {
                super::subagent_status::COMPLETED
            };
            Ok((
                !was_cancelled,
                status.to_string(),
                summary,
                Some(final_msgs),
            ))
        }
        Err(e) if was_cancelled || e == "Annulé" => Ok((
            false,
            super::subagent_status::CANCELLED.to_string(),
            "Sous-agent annulé.".to_string(),
            None,
        )),
        Err(e) if super::subagent_instruction_delivery::is_delivery_error(&e) => Err(e),
        Err(_) => {
            super::stream_diagnostics::record_failure(
                child_session_id,
                Some(request_id),
                "Le sous-agent n'a pas pu terminer correctement.",
                false,
            )
            .await;
            Err("Le sous-agent n'a pas pu terminer correctement.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::{
        session_store, subagent_history, subagent_registry, subagent_status,
    };

    #[tokio::test]
    async fn cancellation_without_snapshot_keeps_durable_history() {
        let parent = session_store::create_full("Parent cancel", "llama3", "ollama", false, None)
            .await
            .expect("create parent");
        let mut child = session_store::create_full("Child cancel", "llama3", "ollama", false, None)
            .await
            .expect("create child");
        child.parent_session_id = Some(parent.id.clone());
        child.subagent_type = Some("explorer".into());
        child.subagent_status = Some(subagent_status::RUNNING.into());
        child.messages.push(super::super::subagent_instruction_delivery::agent_message(
            "mission durable",
        ));
        let registered = subagent_registry::register_execution(
            &parent.id,
            &child.id,
            CancellationToken::new(),
        )
        .await
        .expect("register child");
        child.subagent_run_id = Some(registered.run_id.clone());
        session_store::save(&child).await.expect("save child");
        let cancel = CancellationToken::new();
        cancel.cancel();
        let (_, _, _, snapshot) = finalize_stream_result(
            Err("Annulé".into()),
            &child.id,
            "request",
            cancel,
        )
        .await
        .expect("cancel outcome");

        if let Some(messages) = snapshot.as_deref() {
            subagent_history::persist_for_execution(
                &child.id,
                &registered.run_id,
                &registered.execution_id,
                messages,
            )
            .await
            .expect("persist snapshot");
        }

        let saved = session_store::get(&child.id).await.expect("load child");
        assert!(snapshot.is_none());
        assert_eq!(saved.messages.len(), 1);
        assert_eq!(saved.messages[0].content, "mission durable");
        subagent_registry::unregister(&child.id).await;
        session_store::delete_one(&child.id).await.expect("delete child");
        session_store::delete_one(&parent.id).await.expect("delete parent");
    }
}
