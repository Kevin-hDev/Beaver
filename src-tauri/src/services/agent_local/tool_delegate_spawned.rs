use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub struct SpawnedSubagent {
    pub app: AppHandle,
    pub child_id: String,
    pub model: String,
    pub provider: String,
    pub runtime_context: super::subagent_runtime_context::SubagentRuntimeContext,
    pub prompt: String,
    pub subagent_type: String,
    pub parent_emitter: AgentEventEmitter,
    pub cancel: CancellationToken,
    pub project_id: Option<String>,
    pub run_id: String,
    pub execution_id: String,
    pub spawn_event: StreamEvent,
}
