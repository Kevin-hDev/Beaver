use crate::services::agent_local::stream_events::AgentEventEmitter;
use std::sync::OnceLock;
use tauri::AppHandle;
use tokio::sync::mpsc;

pub struct SpawnRequest {
    pub app: AppHandle,
    pub parent_session_id: String,
    pub child_session_id: String,
    pub model: String,
    pub provider: String,
    pub runtime_context: super::subagent_runtime_context::SubagentRuntimeContext,
    pub prompt: String,
    pub subagent_type: String,
    pub parent_emitter: AgentEventEmitter,
    pub cancel: tokio_util::sync::CancellationToken,
    pub project_id: Option<String>,
    pub run_id: String,
    pub execution_id: String,
}

const MAX_QUEUED: usize = 8;

static TX: OnceLock<mpsc::Sender<SpawnRequest>> = OnceLock::new();

pub fn init() {
    let (tx, rx) = mpsc::channel(MAX_QUEUED);
    let _ = TX.set(tx);
    tauri::async_runtime::spawn(receiver_loop(rx));
}

pub fn send<F>(req: SpawnRequest, after_accepted: F) -> Result<(), String>
where
    F: FnOnce(),
{
    let sender = TX
        .get()
        .ok_or_else(|| "Canal de spawn non initialisé".to_string())?;
    try_send_then(sender, req, after_accepted)
}

fn try_send_then<T, F>(sender: &mpsc::Sender<T>, value: T, after_accepted: F) -> Result<(), String>
where
    F: FnOnce(),
{
    sender
        .try_send(value)
        .map_err(|_| "Trop de sous-agents en attente".to_string())?;
    after_accepted();
    Ok(())
}

async fn receiver_loop(mut rx: mpsc::Receiver<SpawnRequest>) {
    while let Some(req) = rx.recv().await {
        tauri::async_runtime::spawn(async move {
            let parent_session_id = req.parent_session_id.clone();
            let child_session_id = req.child_session_id.clone();
            let subagent_type = req.subagent_type.clone();
            let parent_emitter = req.parent_emitter.clone();
            let run_id = req.run_id.clone();
            let execution_id = req.execution_id.clone();
            if !super::subagent_registry::owns_execution(
                &child_session_id,
                &run_id,
                &execution_id,
            )
            .await
            {
                return;
            }
            let expected_worktree = if req.subagent_type == "coder" {
                let Ok(path) = super::subagent_worktree::path_for_execution(
                    &child_session_id,
                    &execution_id,
                ) else {
                    return;
                };
                Some(path.to_string_lossy().to_string())
            } else {
                None
            };
            let child = super::subagent_task::run(
                req.app,
                req.parent_session_id,
                req.child_session_id,
                req.model,
                req.provider,
                req.runtime_context,
                req.prompt,
                req.subagent_type,
                req.parent_emitter,
                req.cancel,
                req.project_id,
                run_id.clone(),
                execution_id.clone(),
            );
            super::subagent_panic_supervisor::run_guarded(child, move || async move {
                super::subagent_panic_supervisor::recover_panicked_completion(
                    &parent_session_id,
                    &child_session_id,
                    &subagent_type,
                    &run_id,
                    &execution_id,
                    expected_worktree.as_deref(),
                    Some(&parent_emitter),
                )
                .await;
            })
            .await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::try_send_then;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn full_channel_does_not_publish_spawned_event() {
        let (sender, _receiver) = tokio::sync::mpsc::channel(1);
        sender.try_send(1).expect("fill channel");
        let published = AtomicBool::new(false);

        assert!(try_send_then(&sender, 2, || published.store(true, Ordering::SeqCst)).is_err());
        assert!(!published.load(Ordering::SeqCst));
    }

    #[test]
    fn closed_channel_does_not_publish_spawned_event() {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        drop(receiver);
        let published = AtomicBool::new(false);

        assert!(try_send_then(&sender, 1, || published.store(true, Ordering::SeqCst)).is_err());
        assert!(!published.load(Ordering::SeqCst));
    }

    #[test]
    fn accepted_request_publishes_once() {
        let (sender, _receiver) = tokio::sync::mpsc::channel(1);
        let published = AtomicBool::new(false);

        try_send_then(&sender, 1, || published.store(true, Ordering::SeqCst))
            .expect("accept request");
        assert!(published.load(Ordering::SeqCst));
    }
}
