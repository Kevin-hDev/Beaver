#![allow(
    dead_code,
    reason = "the compression orchestrator consumes live state in Task 10"
)]

use serde::Serialize;

use super::session_capabilities::SessionCompressionCapabilities;
use crate::services::agent_local::types_session::AgentSession;

const MAX_FAILURES: usize = 20;

#[derive(Debug, Clone, Serialize)]
pub struct CheckpointLiveState {
    pub git: Vec<crate::services::git::status::DirtyFile>,
    pub todos: Vec<crate::services::agent_local::types_todo::AgentTodoItem>,
    pub active_plan: Option<crate::services::agent_local::types_plan::AgentPlanRun>,
    pub failures: Vec<CheckpointFailure>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckpointFailure {
    pub code: String,
    pub safe_summary: Option<String>,
}

pub fn collect(
    session: &AgentSession,
    capabilities: &SessionCompressionCapabilities,
) -> CheckpointLiveState {
    let git = if capabilities.git {
        crate::services::git::status::list_dirty_files(std::path::Path::new(&session.working_dir))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let (todos, active_plan) = if capabilities.plan_and_tasks {
        let active = session
            .active_plan_id
            .as_deref()
            .and_then(|id| session.plan_runs.iter().find(|run| run.id == id))
            .cloned();
        (session.todos.clone(), active)
    } else {
        (Vec::new(), None)
    };
    let failures = session
        .stream_failures
        .iter()
        .rev()
        .take(MAX_FAILURES)
        .map(|failure| CheckpointFailure {
            code: failure.code.clone(),
            safe_summary: session
                .diagnostic_runs
                .iter()
                .rev()
                .find_map(|run| run.safe_summary.clone()),
        })
        .collect();
    CheckpointLiveState {
        git,
        todos,
        active_plan,
        failures,
    }
}
