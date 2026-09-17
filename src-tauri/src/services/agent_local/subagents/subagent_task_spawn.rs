#![expect(
    clippy::too_many_arguments,
    reason = "allocation boundary preserves the explicit subagent runtime context"
)]

use super::stream_events::AgentEventEmitter;
use super::subagent_runtime_context::SubagentRuntimeContext;
use std::future::Future;
use std::pin::Pin;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) type SpawnedSubagentTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub(super) fn run(
    app: AppHandle,
    parent_session_id: String,
    child_session_id: String,
    model: String,
    provider: String,
    runtime_context: SubagentRuntimeContext,
    prompt: String,
    subagent_type: String,
    parent_emitter: AgentEventEmitter,
    cancel: CancellationToken,
    project_id: Option<String>,
    run_id: String,
    execution_id: String,
) -> SpawnedSubagentTask {
    Box::pin(super::subagent_task::run(
        app,
        parent_session_id,
        child_session_id,
        model,
        provider,
        runtime_context,
        prompt,
        subagent_type,
        parent_emitter,
        cancel,
        project_id,
        run_id,
        execution_id,
    ))
}
