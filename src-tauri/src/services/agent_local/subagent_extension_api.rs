use super::types_session::{
    AgentSession, SubagentExtensionOwner, SubagentExtensionOwnership,
};
use super::types_tools::ToolResult;
use serde_json::json;
use tokio_util::sync::CancellationToken;

pub(crate) async fn spawn(
    parent_id: &str,
    owner: SubagentExtensionOwner,
    kind: &str,
    prompt: &str,
    cancel: CancellationToken,
) -> Result<AgentSession, ToolResult> {
    let pending = super::tool_dispatcher_delegate::spawn_delegate_owned(
        &json!({"subagent_type": kind, "prompt": prompt}),
        parent_id,
        cancel,
        Some(owner),
    )
    .await?;
    super::session_store::get(&pending.child_id)
        .await
        .map_err(|_| ToolResult::internal("subagent_unavailable", "Sous-agent indisponible.", false))
}

pub(crate) async fn list(
    parent_id: &str,
    owner: &SubagentExtensionOwner,
) -> Result<Vec<AgentSession>, ToolResult> {
    let items = super::session_store::list()
        .await
        .map_err(|_| ToolResult::unavailable("subagent_list_unavailable", "Sous-agents indisponibles.", true))?;
    let mut children = Vec::new();
    for item in items
        .into_iter()
        .filter(|item| item.parent_session_id.as_deref() == Some(parent_id))
    {
        if let Ok(child) = super::session_store::get(&item.id).await {
            if owner_matches(&child, parent_id, owner) {
                children.push(child);
            }
        }
    }
    Ok(children)
}

pub(crate) async fn get(
    child_id: &str,
    parent_id: &str,
    owner: &SubagentExtensionOwner,
) -> Result<AgentSession, ToolResult> {
    let child = super::session_store::get(child_id)
        .await
        .map_err(|_| ToolResult::not_found("subagent_not_found", "Sous-agent introuvable."))?;
    owner_matches(&child, parent_id, owner)
        .then_some(child)
        .ok_or_else(|| ToolResult::not_found("subagent_not_found", "Sous-agent introuvable."))
}

pub(crate) async fn send(
    child_id: &str,
    prompt: &str,
    parent_id: &str,
    owner: &SubagentExtensionOwner,
    cancel: CancellationToken,
) -> Result<(), ToolResult> {
    get(child_id, parent_id, owner).await?;
    result(super::tool_subagent_message::run_with_cancel(
        &json!({"subagent_id": child_id, "prompt": prompt}),
        parent_id,
        cancel,
    ).await)
}

pub(crate) async fn cancel(
    child_id: &str,
    parent_id: &str,
    owner: &SubagentExtensionOwner,
) -> Result<bool, ToolResult> {
    get(child_id, parent_id, owner).await?;
    super::subagent_cancellation::cancel_owned(child_id, parent_id)
        .await
        .map_err(|_| ToolResult::internal("subagent_cancel_failed", "Sous-agent indisponible.", false))
}

pub(crate) fn owner_matches(
    child: &AgentSession,
    parent_id: &str,
    owner: &SubagentExtensionOwner,
) -> bool {
    child.parent_session_id.as_deref() == Some(parent_id)
        && matches!(
            child.subagent_extension_owner.as_ref(),
            Some(SubagentExtensionOwnership::Valid(current)) if current == owner
        )
}

fn result(value: ToolResult) -> Result<(), ToolResult> {
    if value.is_error { Err(value) } else { Ok(()) }
}
