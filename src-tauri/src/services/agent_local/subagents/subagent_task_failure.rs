use super::stream_events::AgentEventEmitter;

pub async fn finish_preparation_failure(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    run_id: &str,
    execution_id: &str,
    emitter: Option<&AgentEventEmitter>,
) -> bool {
    let summary = "Le sous-agent n'a pas pu terminer correctement.";
    !matches!(
        super::subagent_completion_events::persist_terminal(
            parent_id,
            child_id,
            subagent_type,
            super::subagent_status::FAILED,
            summary,
            run_id,
            execution_id,
            false,
            emitter,
        )
        .await,
        Ok(None)
    )
}
