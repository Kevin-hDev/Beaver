#![allow(
    dead_code,
    reason = "the compression orchestrator consumes subagent state in Task 10"
)]

use serde::Serialize;
use std::collections::BTreeSet;

use crate::services::agent_local::types_session::{AgentSession, SubagentHiddenReport};

#[derive(Debug, Clone, Serialize)]
pub struct CheckpointSubagents {
    pub active: Vec<ActiveSubagentCheckpoint>,
    pub unreadable_active_ids: Vec<String>,
    pub pending_reports: Vec<ReportCheckpoint>,
    pub delivered_report_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveSubagentCheckpoint {
    pub id: String,
    pub name: String,
    pub subagent_type: String,
    pub status: String,
    pub mission: String,
    pub last_activity: Option<String>,
    pub next_wait: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportCheckpoint {
    pub report_id: String,
    pub child_session_id: String,
    pub name: String,
    pub subagent_type: String,
    pub status: String,
    pub summary: String,
}

pub async fn collect(parent: &AgentSession, detail_tokens: u32) -> CheckpointSubagents {
    let mut remaining_tokens = detail_tokens;
    let mut active_ids =
        crate::services::agent_local::subagent_registry::active_children_for_parent(&parent.id)
            .await;
    active_ids.sort();
    active_ids.truncate(crate::services::agent_local::subagent_registry::MAX_PER_PARENT);
    let mut active = Vec::with_capacity(active_ids.len());
    let mut unreadable_active_ids = Vec::new();
    for child_id in active_ids {
        if let Ok(child) = crate::services::agent_local::session_store::get(&child_id).await {
            active.push(active_checkpoint(&child, &mut remaining_tokens));
        } else {
            unreadable_active_ids.push(child_id);
        }
    }
    let mut seen = BTreeSet::new();
    let mut pending_reports = Vec::new();
    for report in parent
        .subagent_hidden_reports
        .iter()
        .filter(|report| !report.delivered)
    {
        seen.insert(report.id.clone());
        pending_reports.push(report_checkpoint(report, &mut remaining_tokens));
    }
    for report in
        crate::services::agent_local::subagent_report_overflow::pending_for_parent(&parent.id).await
    {
        if seen.insert(report.id.clone()) {
            pending_reports.push(report_checkpoint(&report, &mut remaining_tokens));
        }
    }
    let delivered_report_ids = parent
        .subagent_hidden_reports
        .iter()
        .filter(|report| report.delivered)
        .map(|report| report.id.clone())
        .collect();
    CheckpointSubagents {
        active,
        unreadable_active_ids,
        pending_reports,
        delivered_report_ids,
    }
}

fn active_checkpoint(child: &AgentSession, remaining_tokens: &mut u32) -> ActiveSubagentCheckpoint {
    let activity = child.subagent_last_activity.as_ref();
    ActiveSubagentCheckpoint {
        id: child.id.clone(),
        name: child.name.clone(),
        subagent_type: child
            .subagent_type
            .clone()
            .unwrap_or_else(|| "unknown".into()),
        status: child
            .subagent_status
            .clone()
            .unwrap_or_else(|| "running".into()),
        mission: excerpt(
            child.subagent_prompt.as_deref().unwrap_or_default(),
            remaining_tokens,
        ),
        last_activity: activity
            .map(|value| value.detail.clone().unwrap_or_else(|| value.label.clone())),
        next_wait: if child.subagent_queued_prompts.is_empty() {
            "terminal report".into()
        } else {
            "queued parent instruction".into()
        },
    }
}

fn report_checkpoint(
    report: &SubagentHiddenReport,
    remaining_tokens: &mut u32,
) -> ReportCheckpoint {
    ReportCheckpoint {
        report_id: report.id.clone(),
        child_session_id: report.child_session_id.clone(),
        name: report.name.clone(),
        subagent_type: report.subagent_type.clone(),
        status: report.status.clone(),
        summary: excerpt(&report.summary, remaining_tokens),
    }
}

fn excerpt(value: &str, remaining_tokens: &mut u32) -> String {
    let output = value
        .chars()
        .take(remaining_tokens.saturating_mul(4) as usize)
        .collect::<String>();
    let used = crate::services::token_counting::estimate_text_tokens(&output).min(u32::MAX as usize)
        as u32;
    *remaining_tokens = remaining_tokens.saturating_sub(used);
    output
}
