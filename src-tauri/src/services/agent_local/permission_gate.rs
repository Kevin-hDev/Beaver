use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

pub use super::permission_allow_cache::{clear_session, is_allowed, mark_allowed};

#[derive(Debug, Clone, Copy)]
pub enum PermissionDecision {
    Allow,
    AllowSession,
    Deny,
}

const GATED_TOOLS: &[&str] = &[
    "write_file",
    "edit_file",
    "web_fetch",
    "write_spreadsheet",
    "write_document",
    "process_image",
    "create_branch",
    "checkout_branch",
    "apply_subagent_changes",
    "forecast_data_audit",
    "forecast",
    "forecast_backtest",
];

static SAFE_BASH_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"^ls\b",
        r"^cat\b",
        r"^head\b",
        r"^tail\b",
        r"^wc\b",
        r"^grep\b",
        r"^find\b",
        r"^git\s+(status|log|diff|show|remote|tag)\b",
        r"^git\s+branch\s*$",
        r"^pwd$",
        r"^echo\b",
        r"^which\b",
        r"^cargo\s+(check|test|clippy|build)\b",
        r"^npx\s+tsc\b",
        r"^npm\s+run\b",
        r"^tree\b",
        r"^file\b",
        r"^stat\b",
        r"^du\b",
        r"^df\b",
    ]
    .into_iter()
    .filter_map(|p| Regex::new(p).ok())
    .collect()
});

fn is_safe_bash(command: &str) -> bool {
    let trimmed = command.trim();
    if crate::services::agent_local::sensitive_data::bash_touches_sensitive_data(trimmed) {
        return false;
    }
    if trimmed.contains(';')
        || trimmed.contains("&&")
        || trimmed.contains("||")
        || trimmed.contains('|')
        || trimmed.contains('`')
        || trimmed.contains("$(")
        || trimmed.contains('\n')
        || trimmed.contains('\r')
        || trimmed.contains("<(")
        || trimmed.contains(">(")
        || trimmed.contains("<<")
        || trimmed.contains('>')
        || trimmed.contains("$'")
        || trimmed.contains('&')
        || trimmed.contains('<')
    {
        return false;
    }
    SAFE_BASH_PATTERNS.iter().any(|re| re.is_match(trimmed))
}

pub fn requires_permission(tool_name: &str, args: &serde_json::Value) -> bool {
    match tool_name {
        "bash" => {
            let cmd = args["command"].as_str().unwrap_or("");
            !is_safe_bash(cmd)
        }
        "search_mcp_tools" => args["mode"].as_str() == Some("call"),
        _ => GATED_TOOLS.contains(&tool_name),
    }
}

const MAX_PENDING: usize = 64;
const MAX_DIAGNOSTIC_LOG_BYTES: u64 = 2 * 1024 * 1024;

static PENDING: LazyLock<Mutex<HashMap<String, oneshot::Sender<PermissionDecision>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn log_diagnostic(event: &str, tool_name: Option<&str>, detail: Option<&str>) {
    let entry = diagnostic_entry(event, tool_name, detail);
    eprintln!("[permission] {}", entry);

    let dir = crate::services::paths::data_dir().join("logs");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("permission-diagnostics.jsonl");
    if std::fs::metadata(&path)
        .map(|meta| meta.len() > MAX_DIAGNOSTIC_LOG_BYTES)
        .unwrap_or(false)
    {
        let rotated = dir.join("permission-diagnostics.jsonl.1");
        let _ = std::fs::rename(&path, rotated);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", entry);
    }
}

pub(crate) fn diagnostic_entry(
    event: &str,
    tool_name: Option<&str>,
    detail: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "ts": chrono::Local::now().to_rfc3339(),
        "event": event,
        "tool": tool_name,
        "detail": detail,
    })
}

pub async fn request(
    on_event: &AgentEventEmitter,
    tool_name: &str,
    arguments: &Value,
    cancel: CancellationToken,
) -> PermissionDecision {
    let id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();
    {
        let mut pending = PENDING.lock().await;
        if pending.len() >= MAX_PENDING {
            return PermissionDecision::Deny;
        }
        pending.insert(id.clone(), tx);
    }

    let _ = on_event.send(StreamEvent::PermissionRequest {
        id: id.clone(),
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
    });
    log_diagnostic("request", Some(tool_name), Some("permission_prompt_sent"));

    tokio::select! {
        res = rx => res.unwrap_or(PermissionDecision::Deny),
        _ = cancel.cancelled() => {
            PENDING.lock().await.remove(&id);
            PermissionDecision::Deny
        }
    }
}

pub async fn respond(id: &str, decision: PermissionDecision) {
    if let Some(tx) = PENDING.lock().await.remove(id) {
        let detail = match decision {
            PermissionDecision::Allow => "allow",
            PermissionDecision::AllowSession => "allow_session",
            PermissionDecision::Deny => "deny",
        };
        log_diagnostic("respond_found", None, Some(detail));
        let _ = tx.send(decision);
    } else {
        log_diagnostic("respond_missing", None, Some("stale_or_unknown_permission"));
    }
}
