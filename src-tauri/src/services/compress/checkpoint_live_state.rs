use serde::Serialize;

use super::session_capabilities::SessionCompressionCapabilities;
use crate::services::agent_local::types_session::AgentSession;

const MAX_FAILURES: usize = 20;

#[derive(Debug, Clone, Serialize)]
pub struct CheckpointLiveState {
    pub git: Vec<crate::services::git::status::DirtyFile>,
    pub git_unavailable: bool,
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
    let (git, git_unavailable) = if capabilities.git {
        match crate::services::git::status::list_dirty_files(std::path::Path::new(
            &session.working_dir,
        )) {
            Ok(files) => (files, false),
            Err(_) => (Vec::new(), true),
        }
    } else {
        (Vec::new(), false)
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
            safe_summary: None,
        })
        .collect();
    CheckpointLiveState {
        git,
        git_unavailable,
        todos,
        active_plan,
        failures,
    }
}
