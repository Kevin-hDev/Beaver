use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;

pub struct PendingDelegate {
    child_id: String,
}

pub async fn dispatch_delegate(
    args: &Value,
    session_id: &str,
    cancel: tokio_util::sync::CancellationToken,
) -> ToolResult {
    match spawn_delegate(args, session_id, cancel).await {
        Ok(pending) => pending.wait().await,
        Err(tr) => tr,
    }
}

pub async fn spawn_delegate(
    args: &Value,
    session_id: &str,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<PendingDelegate, ToolResult> {
    let Some(app) = super::app_handle_global::get() else {
        return Err(ToolResult::err("AppHandle non initialisé"));
    };
    let emitter = crate::services::agent_local::stream_events::AgentEventEmitter::new(
        app.clone(),
        session_id.to_string(),
    );
    match super::tool_delegate::prepare_delegate(
        args.clone(),
        app.clone(),
        session_id.to_string(),
        emitter,
        cancel,
    )
    .await
    {
        Err(tr) => Err(tr),
        Ok(spawned) => {
            let child_id = spawned.child_id.clone();
            let spawn_emitter = spawned.parent_emitter.clone();
            let spawn_event = spawned.spawn_event;
            if let Err(e) = super::subagent_spawn_channel::send(
                super::subagent_spawn_channel::SpawnRequest {
                    app: spawned.app,
                    parent_session_id: session_id.to_string(),
                    child_session_id: spawned.child_id,
                    model: spawned.model,
                    provider: spawned.provider,
                    runtime_context: spawned.runtime_context,
                    prompt: spawned.prompt,
                    subagent_type: spawned.subagent_type,
                    parent_emitter: spawned.parent_emitter,
                    cancel: spawned.cancel,
                    project_id: spawned.project_id,
                    run_id: spawned.run_id,
                    execution_id: spawned.execution_id,
                },
                move || {
                    let _ = spawn_emitter.send(spawn_event);
                },
            )
            {
                super::subagent_registry::unregister(&child_id).await;
                if let Err(mark_err) =
                    super::session_subagents::mark_status(&child_id, super::subagent_status::FAILED)
                        .await
                {
                    eprintln!("[delegate] mark_status failed {child_id}: {mark_err}");
                }
                return Err(ToolResult::err(e));
            }
            Ok(PendingDelegate { child_id })
        }
    }
}

impl PendingDelegate {
    pub async fn wait(self) -> ToolResult {
        ToolResult::ok(format!(
            "<subagent id=\"{}\" state=\"running\">\n\
             Sous-agent lancé en session enfant. Le stream parent reste actif jusqu'au rapport. \
             Ne rédige pas de réponse finale avant réception du rapport final.\n\
             </subagent>",
            escape_xml(&self.child_id)
        ))
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
