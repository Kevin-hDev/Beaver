use chrono::Utc;

use super::tool_todo_state;
use super::types_session::AgentSession;
use super::types_todo::{all_completed, AgentTodoRunStatus};

pub const TODO_NEGLECT_ESCALATE_AFTER: u32 = 2;
pub const TODO_NEGLECT_AUTO_PAUSE_AFTER: u32 = 4;
const AUTO_PAUSE_REASON: &str = "auto-pause (negligence)";

pub fn should_escalate(session: &AgentSession) -> bool {
    has_incomplete_active_todo(session) && session.todo_neglect_count >= TODO_NEGLECT_ESCALATE_AFTER
}

pub fn record_user_turn(session: &mut AgentSession) -> bool {
    tool_todo_state::migrate_legacy_todos(session);
    if !has_incomplete_active_todo(session) {
        if all_completed(&session.todos) {
            session.todo_neglect_count = 0;
        }
        return false;
    }
    session.todo_neglect_count = session.todo_neglect_count.saturating_add(1);
    if session.todo_neglect_count < TODO_NEGLECT_AUTO_PAUSE_AFTER {
        return false;
    }
    let Some(index) = tool_todo_state::active_run_index(session) else {
        session.todos.clear();
        session.active_todo_run_id = None;
        session.todo_neglect_count = 0;
        return true;
    };
    tool_todo_state::pause_run(
        &mut session.todo_runs[index],
        Some(AUTO_PAUSE_REASON.to_string()),
        Utc::now(),
    );
    session.todos.clear();
    session.active_todo_run_id = None;
    session.todo_neglect_count = 0;
    true
}

fn has_incomplete_active_todo(session: &AgentSession) -> bool {
    !session.todos.is_empty()
        && !all_completed(&session.todos)
        && session.active_todo_run_id.is_some()
        && session.todo_runs.iter().any(|run| {
            Some(run.id.as_str()) == session.active_todo_run_id.as_deref()
                && run.status == AgentTodoRunStatus::Active
        })
}
