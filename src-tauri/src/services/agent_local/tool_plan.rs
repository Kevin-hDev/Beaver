use super::stream_events::AgentEventEmitter;
use super::types_plan::{
    AgentPlanPreview, AgentPlanRun, AgentPlanStatus, AgentPlanWorkflowStatus, MAX_PLAN_RUNS,
};
use super::types_stream::StreamEvent;
use super::types_tools::ToolResult;
use chrono::Utc;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const MAX_TITLE_CHARS: usize = 120;
const MAX_CONTENT_CHARS: usize = 40_000;

pub async fn set_enabled(session_id: &str, enabled: bool) -> Result<(), String> {
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    session.plan_mode_enabled = enabled;
    session.plan_approval_decision = None;
    if enabled {
        session.plan_workflow_status = AgentPlanWorkflowStatus::NeedsContext;
    } else {
        session.active_plan_id = None;
        session.plan_workflow_status = AgentPlanWorkflowStatus::Cancelled;
    }
    super::session_store::save(&session).await
}

pub async fn is_enabled(session_id: &str) -> bool {
    super::session_store::get(session_id)
        .await
        .map(|session| session.plan_mode_enabled)
        .unwrap_or(false)
}

pub async fn execute(
    args: &Value,
    on_event: &AgentEventEmitter,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    match write_plan(args, on_event, session_id).await {
        Ok(run) => {
            super::tool_plan_approval_request::request_approval(
                on_event, session_id, cancel, &run.title,
            )
            .await
        }
        Err(err) => ToolResult::err(err),
    }
}

pub async fn execute_exit(
    args: &Value,
    on_event: &AgentEventEmitter,
    session_id: &str,
) -> ToolResult {
    let status = match args.get("status").and_then(Value::as_str) {
        Some("approved") => AgentPlanStatus::Approved,
        Some("rejected") => AgentPlanStatus::Rejected,
        _ => return ToolResult::err(super::tool_plan_messages::INVALID_STATUS),
    };
    match exit_plan(session_id, status, on_event).await {
        Ok(()) => ToolResult::ok(super::tool_plan_messages::exited(status)),
        Err(err) => ToolResult::err(err),
    }
}

async fn write_plan(
    args: &Value,
    on_event: &AgentEventEmitter,
    session_id: &str,
) -> Result<AgentPlanRun, String> {
    let title = clean_text(required_str(args, "title")?, MAX_TITLE_CHARS)?;
    let content = clean_text(required_str(args, "content")?, MAX_CONTENT_CHARS)?;
    if title.is_empty() || content.is_empty() {
        return Err(super::tool_plan_messages::INVALID_PLAN.to_string());
    }

    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    if !session.plan_mode_enabled {
        return Err(super::tool_plan_messages::PLAN_INACTIVE.to_string());
    }

    let now = Utc::now();
    let plan_id = session
        .active_plan_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let path = super::tool_plan_storage::plan_path(session_id, &plan_id)?;
    super::tool_plan_storage::write_markdown(&path, &title, &content).await?;

    let run = AgentPlanRun {
        id: plan_id.clone(),
        title: title.clone(),
        status: AgentPlanStatus::AwaitingApproval,
        path: path.to_string_lossy().to_string(),
        created_at: session
            .plan_runs
            .iter()
            .find(|run| run.id == plan_id)
            .map(|run| run.created_at)
            .unwrap_or(now),
        updated_at: now,
    };
    session.active_plan_id = Some(plan_id.clone());
    session.plan_workflow_status = AgentPlanWorkflowStatus::AwaitingApproval;
    session.plan_approval_decision = None;
    super::tool_plan_storage::upsert_run(&mut session.plan_runs, run.clone());
    session.plan_runs.truncate(MAX_PLAN_RUNS);
    super::session_store::save(&session).await?;

    let _ = on_event.send(StreamEvent::PlanPreviewUpdated {
        plan: Some(AgentPlanPreview {
            id: plan_id,
            title,
            content,
            status: AgentPlanStatus::AwaitingApproval,
        }),
    });
    Ok(run)
}

async fn exit_plan(
    session_id: &str,
    status: AgentPlanStatus,
    on_event: &AgentEventEmitter,
) -> Result<(), String> {
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    if !session.plan_mode_enabled {
        return Err(super::tool_plan_messages::PLAN_INACTIVE.to_string());
    }
    super::tool_plan_approval::validate_exit(&session, status)?;
    if let Some(active_id) = session.active_plan_id.clone() {
        if let Some(run) = session.plan_runs.iter_mut().find(|run| run.id == active_id) {
            run.status = status;
            run.updated_at = Utc::now();
        }
    }
    session.plan_workflow_status = match status {
        AgentPlanStatus::Approved => AgentPlanWorkflowStatus::Approved,
        AgentPlanStatus::Rejected => AgentPlanWorkflowStatus::Rejected,
        _ => session.plan_workflow_status,
    };
    session.plan_mode_enabled = false;
    session.active_plan_id = None;
    session.plan_approval_decision = None;
    super::session_store::save(&session).await?;
    let _ = on_event.send(StreamEvent::PlanPreviewUpdated { plan: None });
    let _ = on_event.send(StreamEvent::PlanModeUpdated { enabled: false });
    Ok(())
}

fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| super::tool_plan_messages::INVALID_PLAN.to_string())
}

fn clean_text(value: &str, max_chars: usize) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.chars().count() > max_chars || trimmed.contains('\0') {
        return Err(super::tool_plan_messages::INVALID_PLAN.to_string());
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
#[path = "tool_plan_tests.rs"]
mod tests;
