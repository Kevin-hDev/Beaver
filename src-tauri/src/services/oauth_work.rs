use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkSupervisor,
};
use std::future::Future;
use std::time::Instant;

// Cinq connexions MCP, deux fournisseurs LLM et une connexion Codex peuvent
// coexister ; la somme de ces limites existantes fixe l'unique capacité locale.
pub const MAX_OAUTH_FLOWS: usize = 8;

type OAuthFlowWork = ServiceWorkSupervisor<MAX_OAUTH_FLOWS>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OAuthWorkAdmissionError {
    ShuttingDown,
    Busy,
}

#[derive(Clone)]
pub struct OAuthWorkServices {
    flows: OAuthFlowWork,
}

impl OAuthWorkServices {
    pub fn new(app: AppWorkSupervisor) -> Self {
        Self {
            flows: OAuthFlowWork::new(app),
        }
    }

    pub fn try_admit(
        &self,
    ) -> Result<ServiceWorkAdmission<MAX_OAUTH_FLOWS>, OAuthWorkAdmissionError> {
        self.flows.try_admit().map_err(map_admission_error)
    }

    pub fn spawn<Factory, Task>(&self, work: Factory) -> Result<(), OAuthWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.flows.spawn(work).map_err(map_admission_error)
    }

    pub async fn run<Factory, Task, Output>(
        &self,
        work: Factory,
    ) -> Result<Output, OAuthWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future<Output = Output> + Send + 'static,
        Output: Send + 'static,
    {
        let admission = self.try_admit()?;
        let (sender, receiver) = tokio::sync::oneshot::channel();
        admission
            .spawn(move |cancel| async move {
                let output = work(cancel).await;
                let _ = sender.send(output);
            })
            .map_err(map_admission_error)?;
        receiver
            .await
            .map_err(|_| OAuthWorkAdmissionError::ShuttingDown)
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.flows.stop_and_wait(deadline).await
    }
}

fn map_admission_error(error: ServiceWorkAdmissionError) -> OAuthWorkAdmissionError {
    match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => {
            OAuthWorkAdmissionError::ShuttingDown
        }
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            OAuthWorkAdmissionError::Busy
        }
    }
}
