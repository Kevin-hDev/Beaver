use crate::services::agent_local::stream_events::AgentEventEmitter;
use serde_json::Value;
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

pub fn automation_permission_key(tool_name: &str, args: &Value) -> Option<&'static str> {
    if tool_name != "manage_automation" {
        return None;
    }
    match args["action"].as_str() {
        Some("list" | "get" | "history") => Some("manage_automation:read"),
        _ => Some("manage_automation:mutate"),
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

const MAX_DIAGNOSTIC_LOG_BYTES: u64 = 2 * 1024 * 1024;

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
    request_with_deadline(on_event, tool_name, arguments, cancel, None).await
}

async fn request_with_deadline(
    on_event: &AgentEventEmitter,
    tool_name: &str,
    arguments: &Value,
    cancel: CancellationToken,
    deadline: Option<std::time::Instant>,
) -> PermissionDecision {
    let id = uuid::Uuid::new_v4().to_string();
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
    log_diagnostic("request", Some(tool_name), Some("permission_prompt_sent"));
    super::permission_pending::wait(on_event, request, cancel, deadline).await
}

pub(crate) async fn request_extension_core(
    on_event: &AgentEventEmitter,
    tool_name: &str,
    method: &str,
    effect: crate::services::extensions::ExtensionEffect,
    arguments: &Value,
    cancel: CancellationToken,
    deadline: std::time::Instant,
) -> PermissionDecision {
    let Some(indexed) = crate::services::extensions::indexed_tool(tool_name) else {
        return PermissionDecision::Deny;
    };
    let id = uuid::Uuid::new_v4().to_string();
    let request = super::permission_request::for_extension(
        id.clone(),
        &indexed.extension_id,
        &indexed.extension_name,
        method,
        effect,
        arguments,
    );
    super::permission_pending::wait(on_event, request, cancel, Some(deadline)).await
}

pub async fn respond(id: &str, decision: PermissionDecision) {
    if super::permission_pending::respond(id, decision) {
        let detail = match decision {
            PermissionDecision::Allow => "allow",
            PermissionDecision::AllowSession => "allow_session",
            PermissionDecision::Deny => "deny",
        };
        log_diagnostic("respond_found", None, Some(detail));
    } else {
        log_diagnostic("respond_missing", None, Some("stale_or_unknown_permission"));
    }
}
