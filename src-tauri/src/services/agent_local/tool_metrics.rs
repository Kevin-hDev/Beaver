use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tokio::sync::Mutex;

use super::tool_result_contract::{ToolErrorCategory, ToolResultStatus};
use super::types_tools::ToolResult;

pub(super) const MAX_TRACKED_TOOLS: usize = 256;
const MAX_REPORT_TOOLS: usize = 20;
static STORE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMetricErrors {
    pub validation: u64,
    pub permission: u64,
    pub not_found: u64,
    pub conflict: u64,
    pub timeout: u64,
    pub cancelled: u64,
    pub unavailable: u64,
    pub external: u64,
    pub execution: u64,
    pub internal: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMetricEntry {
    pub name: String,
    pub invocations: u64,
    pub success: u64,
    pub running: u64,
    pub partial: u64,
    pub failed: u64,
    pub cancelled: u64,
    pub stopped: u64,
    pub user_denied: u64,
    pub policy_blocked: u64,
    pub errors: ToolMetricErrors,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMetricReport {
    tracked_tools: usize,
    total_invocations: u64,
    tools: Vec<ToolMetricEntry>,
}

pub async fn record(name: &str, result: &ToolResult) -> Result<(), String> {
    if cfg!(test) {
        return Ok(());
    }
    validate_name(name)?;
    let _guard = STORE_LOCK.lock().await;
    let mut entries = super::tool_metrics_store::load().await?;
    update_entries(&mut entries, name, result, chrono::Utc::now().timestamp());
    super::tool_metrics_store::save(&entries).await
}

pub async fn summary(limit: usize) -> Result<ToolMetricReport, String> {
    let _guard = STORE_LOCK.lock().await;
    let entries = super::tool_metrics_store::load().await?;
    Ok(build_report(entries, limit))
}

fn build_report(mut entries: Vec<ToolMetricEntry>, limit: usize) -> ToolMetricReport {
    let tracked_tools = entries.len();
    let total_invocations = entries
        .iter()
        .fold(0_u64, |total, entry| total.saturating_add(entry.invocations));
    entries.sort_by(|left, right| {
        right
            .failed
            .saturating_add(right.cancelled)
            .cmp(&left.failed.saturating_add(left.cancelled))
            .then_with(|| right.invocations.cmp(&left.invocations))
            .then_with(|| left.name.cmp(&right.name))
    });
    entries.truncate(limit.clamp(1, MAX_REPORT_TOOLS));
    ToolMetricReport {
        tracked_tools,
        total_invocations,
        tools: entries,
    }
}

fn update_entries(entries: &mut Vec<ToolMetricEntry>, name: &str, result: &ToolResult, now: i64) {
    if let Some(entry) = entries.iter_mut().find(|entry| entry.name == name) {
        update_entry(entry, result, now);
        return;
    }
    if entries.len() >= MAX_TRACKED_TOOLS {
        let index = entries
            .iter()
            .enumerate()
            .min_by_key(|(_, entry)| (entry.updated_at, entry.invocations))
            .map(|(index, _)| index)
            .unwrap_or(0);
        entries.remove(index);
    }
    let mut entry = ToolMetricEntry {
        name: name.to_string(),
        ..Default::default()
    };
    update_entry(&mut entry, result, now);
    entries.push(entry);
}

fn update_entry(entry: &mut ToolMetricEntry, result: &ToolResult, now: i64) {
    entry.invocations = entry.invocations.saturating_add(1);
    entry.updated_at = now;
    let counter = match result.status {
        ToolResultStatus::Success => &mut entry.success,
        ToolResultStatus::Running => &mut entry.running,
        ToolResultStatus::Partial => &mut entry.partial,
        ToolResultStatus::Error => &mut entry.failed,
        ToolResultStatus::Cancelled => &mut entry.cancelled,
        ToolResultStatus::Stopped => &mut entry.stopped,
    };
    *counter = counter.saturating_add(1);
    let Some(error) = result.error.as_ref() else {
        return;
    };
    error_counter(&mut entry.errors, error.category);
    let code = error.code.as_ref();
    if error.category == ToolErrorCategory::Permission && code == "user_denied_tool" {
        entry.user_denied = entry.user_denied.saturating_add(1);
    } else if error.category == ToolErrorCategory::Permission && is_policy_block(code) {
        entry.policy_blocked = entry.policy_blocked.saturating_add(1);
    }
}

fn is_policy_block(code: &str) -> bool {
    matches!(
        code,
        "memory_operation_forbidden"
            | "memory_path_denied"
            | "memory_read_disabled"
            | "memory_target_invalid"
            | "memory_write_not_authorized"
            | "memory_write_policy_failed"
            | "nested_subagent_control_forbidden"
            | "nested_subagent_delegation_forbidden"
            | "shell_command_blocked"
            | "symlink_write_denied"
            | "tool_disabled"
            | "tool_hook_denied"
            | "tool_not_allowed_for_session"
            | "tool_not_allowed_in_plan"
            | "web_fetch_url_blocked"
            | "write_guard_rejected"
            | "write_path_denied"
    )
}

fn error_counter(errors: &mut ToolMetricErrors, category: ToolErrorCategory) {
    let counter = match category {
        ToolErrorCategory::Validation => &mut errors.validation,
        ToolErrorCategory::Permission => &mut errors.permission,
        ToolErrorCategory::NotFound => &mut errors.not_found,
        ToolErrorCategory::Conflict => &mut errors.conflict,
        ToolErrorCategory::Timeout => &mut errors.timeout,
        ToolErrorCategory::Cancelled => &mut errors.cancelled,
        ToolErrorCategory::Unavailable => &mut errors.unavailable,
        ToolErrorCategory::External => &mut errors.external,
        ToolErrorCategory::Execution => &mut errors.execution,
        ToolErrorCategory::Internal => &mut errors.internal,
    };
    *counter = counter.saturating_add(1);
}

pub(super) fn validate_name(name: &str) -> Result<(), String> {
    crate::services::extensions::validate_identifier(name)
        .map_err(|_| "Mesures d'outils indisponibles.".to_string())
}

#[cfg(test)]
#[path = "tool_metrics_tests.rs"]
mod tests;
