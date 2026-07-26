use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;

pub(super) async fn persist_terminal(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    status: &str,
    summary: &str,
    run_id: &str,
    execution_id: &str,
    success: bool,
    emitter: Option<&AgentEventEmitter>,
) -> Result<Option<super::subagent_task::FinalizedSubagent>, String> {
    let after_report = terminal_callback(emitter, child_id, run_id);
    super::subagent_completion::persist_terminal_completion_inner(
        parent_id,
        child_id,
        subagent_type,
        status,
        summary,
        success,
        Some((run_id, execution_id)),
        || async {},
        after_report,
    )
    .await
}

pub(super) async fn persist_instruction_failure(
    parent_id: &str,
    child_id: &str,
    subagent_type: &str,
    run_id: &str,
    execution_id: &str,
    emitter: Option<&AgentEventEmitter>,
) -> Result<bool, String> {
    let after_report = terminal_callback(emitter, child_id, run_id);
    super::subagent_completion::persist_instruction_delivery_failure_inner(
        parent_id,
        child_id,
        subagent_type,
        Some((run_id, execution_id)),
        after_report,
    )
    .await
}

fn terminal_callback(
    emitter: Option<&AgentEventEmitter>,
    child_id: &str,
    run_id: &str,
) -> impl FnOnce(super::subagent_completion::TerminalOutcome) -> std::future::Ready<()> {
    let emitter = emitter.cloned();
    let child_id = child_id.to_string();
    let run_id = run_id.to_string();
    move |outcome| {
        if let Some(emitter) = emitter {
            let _ = emitter.send(StreamEvent::SubagentCompleted {
                subagent_session_id: child_id,
                success: outcome.success,
                status: outcome.status,
                summary: outcome.summary,
                run_id: Some(run_id),
            });
        }
        std::future::ready(())
    }
}
