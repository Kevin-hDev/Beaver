#[cfg(test)]
use super::subagent_instruction_delivery::{MAX_PROMPT_SIZE, MAX_QUEUED_PROMPTS};
#[cfg(test)]
use super::tool_subagent_message::{build_resume_payload, enqueue_prompt};
#[cfg(test)]
use super::tool_subagent_message::run as message;
use super::tool_subagent_format::{format_child, format_meta};
use super::types_session::AgentSession;
use super::types_tools::ToolResult;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    parent_id: &str,
    stream_cancel: CancellationToken,
) -> Option<ToolResult> {
    if is_child_session(parent_id).await {
        return Some(ToolResult::err(
            "Les sous-agents ne peuvent pas piloter d'autres sous-agents.",
        ));
    }
    Some(match tool_name {
        "list_subagents" => list(parent_id).await,
        "get_subagent" => get(args, parent_id).await,
        "cancel_subagent" => cancel(args, parent_id).await,
        "message_subagent" => {
            super::tool_subagent_message::run_with_cancel(args, parent_id, stream_cancel).await
        }
        "archive_subagent" => archive(args, parent_id).await,
        _ => return None,
    })
}

async fn list(parent_id: &str) -> ToolResult {
    match super::session_store::list().await {
        Ok(items) => {
            let mut rows = Vec::new();
            for item in items
                .into_iter()
                .filter(|item| item.parent_session_id.as_deref() == Some(parent_id))
            {
                rows.push(format_meta(
                    super::subagent_live_state::normalize_meta(item).await,
                ));
            }
            ToolResult::ok(if rows.is_empty() {
                "Aucun sous-agent pour cette session.".to_string()
            } else {
                rows.join("\n")
            })
        }
        Err(_) => ToolResult::err("Sous-agents indisponibles."),
    }
}

async fn get(args: &Value, parent_id: &str) -> ToolResult {
    match owned_child(args, parent_id).await {
        Ok(child) => ToolResult::ok(format_child(
            &super::subagent_live_state::normalize_session(child).await,
        )),
        Err(result) => result,
    }
}

async fn cancel(args: &Value, parent_id: &str) -> ToolResult {
    let Some(child_id) = args["subagent_id"].as_str() else {
        return ToolResult::err("Sous-agent introuvable.");
    };
    match super::subagent_cancellation::cancel_owned(child_id, parent_id).await {
        Ok(true) => ToolResult::ok("Sous-agent annulé.".to_string()),
        Ok(false) => ToolResult::ok("Sous-agent déjà terminé.".to_string()),
        Err(_) => ToolResult::err("Sous-agent indisponible."),
    }
}

async fn archive(args: &Value, parent_id: &str) -> ToolResult {
    let Some(child_id) = args["subagent_id"].as_str() else {
        return ToolResult::err("Sous-agent introuvable.");
    };
    match super::subagent_archive::archive_owned(child_id, parent_id).await {
        Ok(super::subagent_archive::ArchiveOutcome::Archived) => {
            ToolResult::ok("Sous-agent archivé.".to_string())
        }
        Ok(super::subagent_archive::ArchiveOutcome::Active) => {
            ToolResult::err("Sous-agent encore actif.")
        }
        Ok(super::subagent_archive::ArchiveOutcome::NotFound) => {
            ToolResult::err("Sous-agent introuvable.")
        }
        Err(_) => ToolResult::err("Sous-agent indisponible."),
    }
}

#[cfg(test)]
fn can_archive_child(has_pending_work: bool) -> Result<(), ToolResult> {
    if has_pending_work {
        return Err(ToolResult::err("Sous-agent encore actif."));
    }
    Ok(())
}

async fn owned_child(args: &Value, parent_id: &str) -> Result<AgentSession, ToolResult> {
    let Some(id) = args["subagent_id"].as_str() else {
        return Err(ToolResult::err("Sous-agent introuvable."));
    };
    owned_child_by_id(id, parent_id).await
}

pub(super) async fn owned_child_by_id(
    id: &str,
    parent_id: &str,
) -> Result<AgentSession, ToolResult> {
    let child = super::session_store::get(id)
        .await
        .map_err(|_| ToolResult::err("Sous-agent introuvable."))?;
    if !is_owned_by_parent(&child, parent_id) {
        return Err(ToolResult::err("Sous-agent introuvable."));
    }
    Ok(child)
}

fn is_owned_by_parent(child: &AgentSession, parent_id: &str) -> bool {
    child.parent_session_id.as_deref() == Some(parent_id)
}

async fn is_child_session(session_id: &str) -> bool {
    super::session_store::get(session_id)
        .await
        .map(|session| session.parent_session_id.is_some())
        .unwrap_or(false)
}

#[cfg(test)]
async fn child_has_pending_work(child: &AgentSession) -> bool {
    super::subagent_live_state::has_pending_work(child).await
}

#[cfg(test)]
#[path = "tool_subagent_control_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "tool_subagent_message_tests.rs"]
mod message_tests;
#[cfg(test)]
#[path = "tool_subagent_terminal_message_tests.rs"]
mod terminal_message_tests;
