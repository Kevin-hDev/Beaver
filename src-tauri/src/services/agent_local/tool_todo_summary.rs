use super::tool_todo_state::completed_count;
use super::types_session::AgentSession;
use super::types_todo::{AgentTodoItem, AgentTodoRun, AgentTodoRunStatus, AgentTodoStatus};

pub fn history_summary(session: &AgentSession) -> String {
    if session.todo_runs.is_empty() {
        return "Aucune todo list sauvegardée.".to_string();
    }
    session
        .todo_runs
        .iter()
        .rev()
        .map(format_run_summary)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn reminder(session: &AgentSession) -> Option<String> {
    let active = active_run(session);
    let paused = paused_runs(session);
    let failure = session.stream_failures.last();
    if active.is_none() && paused.is_empty() && failure.is_none() {
        return None;
    }

    let mut lines = vec!["\n\n## Todo memory".to_string()];
    if let Some(run) = active {
        lines.extend(active_summary(
            run,
            session.todo_neglect_count,
            super::tool_todo_neglect::should_escalate(session),
        ));
    }
    if !paused.is_empty() {
        lines.push(format!("Paused todos ({}):", paused.len()));
        lines.extend(paused.iter().map(|run| {
            let reason = run
                .paused_reason
                .as_ref()
                .map(|value| format!(" reason=\"{value}\""))
                .unwrap_or_default();
            format!(
                "- id={} title=\"{}\" progress={}/{}{}",
                run.id,
                run.title,
                completed_count(&run.todos),
                run.todos.len(),
                reason
            )
        }));
        lines.push(
            "Use todo_history and todo_resume when a paused todo becomes relevant again."
                .to_string(),
        );
    }
    if let Some(err) = failure {
        lines.push(format!(
            "Last stream diagnostic: {}. Use agent_diagnostics if you need details.",
            err.code
        ));
    }
    Some(lines.join("\n"))
}

fn active_run(session: &AgentSession) -> Option<&AgentTodoRun> {
    let active_id = session.active_todo_run_id.as_ref()?;
    session
        .todo_runs
        .iter()
        .find(|run| &run.id == active_id && run.status == AgentTodoRunStatus::Active)
}

fn paused_runs(session: &AgentSession) -> Vec<&AgentTodoRun> {
    session
        .todo_runs
        .iter()
        .rev()
        .filter(|run| run.status == AgentTodoRunStatus::Paused)
        .collect()
}

fn active_summary(run: &AgentTodoRun, neglect_count: u32, escalate: bool) -> Vec<String> {
    let mut lines = vec![format!(
        "Active todo: id={} title=\"{}\" progress={}/{}.",
        run.id,
        run.title,
        completed_count(&run.todos),
        run.todos.len()
    )];
    lines.extend(run.todos.iter().map(format_todo_item));
    if escalate {
        lines.push(format!(
            "Important: this active todo has not been updated for {neglect_count} user turns. Call todo_write to mark progress, todo_pause if you changed focus, or todo_delete with active=true only if it is obsolete."
        ));
    }
    lines
}

fn format_todo_item(todo: &AgentTodoItem) -> String {
    format!("- [{}] {}", status_label(todo.status), todo.content)
}

fn status_label(status: AgentTodoStatus) -> &'static str {
    match status {
        AgentTodoStatus::Pending => "pending",
        AgentTodoStatus::InProgress => "in_progress",
        AgentTodoStatus::Completed => "completed",
    }
}

fn format_run_summary(run: &AgentTodoRun) -> String {
    let reason = run
        .paused_reason
        .as_ref()
        .map(|value| format!(" reason=\"{value}\""))
        .unwrap_or_default();
    format!(
        "- id={} status={:?} progress={}/{} title=\"{}\"{}",
        run.id,
        run.status,
        completed_count(&run.todos),
        run.todos.len(),
        run.title,
        reason
    )
}
