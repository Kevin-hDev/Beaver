use crate::services::agent_local::agent_work_supervision::{
    AgentWorkServices, MAX_ACTIVE_AGENT_STREAMS,
};
use crate::services::work_registry::{ServiceWorkAdmission, ServiceWorkAdmissionError};
use std::future::Future;
use std::pin::Pin;
use tokio_util::sync::CancellationToken;

pub type AgentStreamAdmission = ServiceWorkAdmission<MAX_ACTIVE_AGENT_STREAMS>;
pub type SpawnedAgentWork = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub fn admit(work: &AgentWorkServices) -> Result<AgentStreamAdmission, String> {
    work.streams().try_admit().map_err(public_error)
}

pub fn spawn(
    admission: AgentStreamAdmission,
    request_cancel: CancellationToken,
    task: SpawnedAgentWork,
) -> Result<(), String> {
    admission
        .spawn(move |shutdown| async move {
            tokio::pin!(task);
            tokio::select! {
                _ = shutdown.cancelled() => {
                    request_cancel.cancel();
                    task.await;
                }
                _ = &mut task => {}
            }
        })
        .map_err(public_error)
}

fn public_error(error: ServiceWorkAdmissionError) -> String {
    error.public_code().to_string()
}
