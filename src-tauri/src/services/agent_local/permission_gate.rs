use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

pub async fn clear_session(session_id: &str) {
    super::permission_allow_cache::clear_session(session_id).await;
}

pub async fn is_allowed(session_id: &str, tool_name: &str) -> bool {
    match crate::services::extensions::indexed_tool(tool_name) {
        Some(indexed) => {
            super::permission_allow_cache::is_extension_allowed(
                session_id,
                &indexed.extension_id,
                tool_name,
            )
            .await
        }
        None => super::permission_allow_cache::is_allowed(session_id, tool_name).await,
    }
}

pub async fn mark_allowed(session_id: &str, tool_name: &str) {
    match crate::services::extensions::indexed_tool(tool_name) {
        Some(indexed)
            if super::permission_policy::extension_effect_policy(indexed.tool.effect)
                .allow_session_cache =>
        {
            super::permission_allow_cache::mark_extension_allowed(
                session_id,
                &indexed.extension_id,
                tool_name,
            )
            .await;
        }
        Some(_) => {}
        None => super::permission_allow_cache::mark_allowed(session_id, tool_name).await,
    }
}

pub(crate) async fn clear_extension(extension_id: &str) {
    super::permission_allow_cache::clear_extension(extension_id).await;
}

pub(crate) async fn clear_all_extensions() {
    super::permission_allow_cache::clear_all_extensions().await;
}

pub(crate) async fn is_extension_allowed(
    session_id: &str,
    extension_id: &str,
    tool_name: &str,
) -> bool {
    super::permission_allow_cache::is_extension_allowed(session_id, extension_id, tool_name).await
}

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
    "create_branch",
    "checkout_branch",
    "apply_subagent_changes",
    "manage_automation",
    "forecast_data_audit",
    "forecast_run",
    "forecast_backtest",
];

pub fn requires_permission(tool_name: &str, args: &serde_json::Value) -> bool {
    if let Some(indexed) = crate::services::extensions::indexed_tool(tool_name) {
        // external-read réutilise la décision et le dialogue de web_fetch, jamais son filtre
        // anti-SSRF : le code Node approuvé garde son accès réseau direct.
        return super::permission_policy::extension_effect_policy(indexed.tool.effect)
            .requires_confirmation;
    }
    match tool_name {
        "bash" => {
            let cmd = args["command"].as_str().unwrap_or("");
            !super::permission_bash::is_safe(cmd)
        }
        "bash_control" => args["chars"]
            .as_str()
            .is_some_and(|input| !input.is_empty()),
        "transform_image" => !args["operations"].as_array().is_some_and(Vec::is_empty),
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
    ::log::info!("[permission] {}", entry);

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

    let request = crate::services::extensions::indexed_tool(tool_name).map_or_else(
        || super::permission_request::native(id.clone(), tool_name, arguments),
        |indexed| {
            super::permission_request::for_extension(
                id.clone(),
                &indexed.extension_id,
                &indexed.extension_name,
                tool_name,
                indexed.tool.effect,
                arguments,
            )
        },
    );
    let _ = on_event.send(StreamEvent::PermissionRequest(request));
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
