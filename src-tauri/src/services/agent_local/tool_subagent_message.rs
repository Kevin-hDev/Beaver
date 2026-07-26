use super::subagent_instruction_delivery::{EnqueueOutcome, MAX_PROMPT_SIZE};
use super::types_session::AgentSession;
use super::types_tools::ToolResult;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

#[cfg(test)]
pub(super) async fn run(args: &Value, parent_id: &str) -> ToolResult {
    run_with_cancel(args, parent_id, CancellationToken::new()).await
}

pub(super) async fn run_with_cancel(
    args: &Value,
    parent_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    let Some(prompt) = valid_prompt(args) else {
        return ToolResult::err("Instruction sous-agent invalide.");
    };
    let Some(child_id) = valid_child_id(args) else {
        return ToolResult::err("Sous-agent introuvable.");
    };
    let payload = {
        let lock = super::session_store::lock_session(child_id).await;
        let _guard = lock.lock().await;
        let Ok(mut child) = super::tool_subagent_control::owned_child_by_id(child_id, parent_id).await
        else {
            return ToolResult::err("Sous-agent introuvable.");
        };
        let active_run = super::subagent_registry::active_run_for_child(&child.id).await;
        if let Some(active) = active_run.as_ref().filter(|run| {
            !run.cancelled && child.subagent_run_id.as_deref() == Some(&run.run_id)
        }) {
            if super::subagent_registry::prompt_was_delivered(
                &child.id,
                &active.execution_id,
                prompt,
            )
            .await
            {
                return duplicate_result();
            }
            return enqueue_live(&mut child, prompt, &active.execution_id).await;
        }
        if active_run.is_some()
            || child.subagent_status.as_deref() != Some(super::subagent_status::COMPLETED)
        {
            return redeploy_required();
        }
        match build_resume_payload(&child, prompt) {
            Ok(payload) => payload,
            Err(result) => return result,
        }
    };
    super::tool_dispatcher_delegate::dispatch_delegate(&payload, parent_id, cancel).await
}

fn valid_prompt(args: &Value) -> Option<&str> {
    args["prompt"].as_str().filter(|value| {
        !value.trim().is_empty() && value.chars().count() <= MAX_PROMPT_SIZE
    })
}

fn valid_child_id(args: &Value) -> Option<&str> {
    let id = args["subagent_id"].as_str()?.trim();
    (!id.is_empty() && super::session_store::validate_session_id(id).is_ok()).then_some(id)
}

async fn enqueue_live(child: &mut AgentSession, prompt: &str, execution_id: &str) -> ToolResult {
    match enqueue_prompt(child, prompt) {
        Ok(EnqueueOutcome::Duplicate) => duplicate_result(),
        Ok(EnqueueOutcome::Added) => {
            match super::subagent_registry::save_queued_prompt_for_execution(child, execution_id)
                .await
            {
                Ok(()) => {
                    ToolResult::ok("Instruction ajoutée à la file du sous-agent.".to_string())
                }
                Err(_) => ToolResult::err("Sous-agent indisponible ou limite atteinte."),
            }
        }
        Err(result) => result,
    }
}

fn duplicate_result() -> ToolResult {
    ToolResult::ok("Instruction déjà présente ou livrée; aucun doublon ajouté.".to_string())
}

fn redeploy_required() -> ToolResult {
    ToolResult::err(
        "Sous-agent terminé. Utilisez delegate_task avec subagent_id pour le redéployer.",
    )
}

pub(super) fn build_resume_payload(
    child: &AgentSession,
    prompt: &str,
) -> Result<Value, ToolResult> {
    let subagent_type = match child.subagent_type.as_deref() {
        Some(value @ ("explorer" | "coder")) => value,
        _ => return Err(redeploy_required()),
    };
    Ok(json!({
        "subagent_id": child.id,
        "subagent_type": subagent_type,
        "display_name": child.name,
        "description": child.subagent_description.as_deref().unwrap_or(""),
        "prompt": prompt,
    }))
}

pub(super) fn enqueue_prompt(
    child: &mut AgentSession,
    prompt: &str,
) -> Result<EnqueueOutcome, ToolResult> {
    super::subagent_instruction_delivery::enqueue(child, prompt)
}
