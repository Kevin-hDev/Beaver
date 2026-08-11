use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation,
};
use std::future::Future;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::agent_work_supervision::{AgentWorkServices, MAX_ACTIVE_SUBAGENTS};

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

type SubagentAdmission = ServiceWorkAdmission<MAX_ACTIVE_SUBAGENTS>;

struct QueuedSpawn {
    request: SpawnRequest,
    admission: SubagentAdmission,
}

static TX: OnceLock<mpsc::Sender<QueuedSpawn>> = OnceLock::new();

pub fn init(app: &AppHandle) -> Result<(), String> {
    let (tx, rx) = mpsc::channel(MAX_QUEUED);
    TX.set(tx)
        .map_err(|_| "subagent-dispatcher-already-initialized".to_string())?;
    let dispatcher = app.state::<AgentWorkServices>().subagent_dispatcher();
    dispatcher
        .spawn(move |shutdown| receiver_loop(rx, shutdown))
        .map_err(public_error)
}

pub fn send<F>(req: SpawnRequest, after_accepted: F) -> Result<(), String>
where
    F: FnOnce(),
{
    let admission = req
        .app
        .state::<AgentWorkServices>()
        .subagents()
        .try_admit()
        .map_err(public_error)?;
    let sender = TX
        .get()
        .ok_or_else(|| "Canal de spawn non initialisé".to_string())?;
    try_send_then(
        sender,
        QueuedSpawn {
            request: req,
            admission,
        },
        after_accepted,
    )
}

pub(super) fn try_send_then<T, F>(
    sender: &mpsc::Sender<T>,
    value: T,
    after_accepted: F,
) -> Result<(), String>
where
    F: FnOnce(),
{
    sender
        .try_send(value)
        .map_err(|_| "Trop de sous-agents en attente".to_string())?;
    after_accepted();
    Ok(())
}

async fn receiver_loop(mut rx: mpsc::Receiver<QueuedSpawn>, shutdown: ServiceWorkCancellation) {
    while let Some(queued) = receive_next(&mut rx, &shutdown).await {
        let req = queued.request;
        let admission = queued.admission;
        let request_cancel = req.cancel.clone();
        let refused_child_id = req.child_session_id.clone();
        let refused_cancel = req.cancel.clone();
        let task = async move {
            let parent_session_id = req.parent_session_id.clone();
            let child_session_id = req.child_session_id.clone();
            let subagent_type = req.subagent_type.clone();
            let parent_emitter = req.parent_emitter.clone();
            let run_id = req.run_id.clone();
            let execution_id = req.execution_id.clone();
            if !super::subagent_registry::owns_execution(&child_session_id, &run_id, &execution_id)
                .await
            {
                return;
            }
            let expected_worktree = if req.subagent_type == "coder" {
                let Ok(path) =
                    super::subagent_worktree::path_for_execution(&child_session_id, &execution_id)
                else {
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
        };
        if spawn_tracked(admission, request_cancel, task).is_err() {
            cancel_unstarted(&refused_child_id, &refused_cancel).await;
            ::log::warn!("[subagent] travail refusé pendant la fermeture");
        }
    }
    while let Some(queued) = rx.recv().await {
        cancel_unstarted(&queued.request.child_session_id, &queued.request.cancel).await;
    }
}

async fn cancel_unstarted(child_id: &str, cancel: &CancellationToken) {
    if !super::subagent_cancellation::cancel(child_id)
        .await
        .unwrap_or(false)
    {
        cancel.cancel();
    }
}

pub(super) async fn receive_next<T>(
    receiver: &mut mpsc::Receiver<T>,
    shutdown: &ServiceWorkCancellation,
) -> Option<T> {
    tokio::select! {
        biased;
        _ = shutdown.cancelled() => {
            receiver.close();
            None
        }
        value = receiver.recv() => value,
    }
}

pub(super) fn spawn_tracked<Task>(
    admission: SubagentAdmission,
    request_cancel: CancellationToken,
    task: Task,
) -> Result<(), ServiceWorkAdmissionError>
where
    Task: Future<Output = ()> + Send + 'static,
{
    admission.spawn(move |shutdown| async move {
        tokio::pin!(task);
        tokio::select! {
            _ = shutdown.cancelled() => {
                request_cancel.cancel();
                task.await;
            }
            _ = &mut task => {}
        }
    })
}

fn public_error(error: ServiceWorkAdmissionError) -> String {
    error.public_code().to_string()
}
