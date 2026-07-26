use super::types_session::AgentSession;

pub(super) async fn loaded(
    child: &AgentSession,
    expected_owner: Option<(&str, &str)>,
) -> bool {
    let Some((run_id, execution_id)) = expected_owner else {
        return true;
    };
    child.subagent_run_id.as_deref() == Some(run_id)
        && super::subagent_registry::owns_execution(&child.id, run_id, execution_id).await
}

pub(super) async fn missing(
    child_id: &str,
    expected_owner: Option<(&str, &str)>,
) -> bool {
    let Some((run_id, execution_id)) = expected_owner else {
        return true;
    };
    super::subagent_registry::owns_execution(child_id, run_id, execution_id).await
}
