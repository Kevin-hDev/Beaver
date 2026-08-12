use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkSupervisor,
};
use std::future::Future;
use std::time::Instant;

// Chaque flux agent peut attendre le même démarrage sérialisé ; réutiliser sa
// capacité évite une seconde limite concurrente pour les recherches.
const START_OPERATIONS: usize =
    crate::services::agent_local::agent_work_supervision::MAX_ACTIVE_AGENT_STREAMS;
pub(super) const SERVER_PROCESSES: usize = 1;

type StartWork = ServiceWorkSupervisor<START_OPERATIONS>;
type ServerWork = ServiceWorkSupervisor<SERVER_PROCESSES>;

#[derive(Clone)]
pub(super) struct SearxngWorkServices {
    starts: StartWork,
    server: ServerWork,
}

impl SearxngWorkServices {
    pub(super) fn new(app: AppWorkSupervisor) -> Self {
        Self {
            starts: StartWork::new(app.clone()),
            server: ServerWork::new(app),
        }
    }

    pub(super) fn spawn_start<Factory, Task>(&self, work: Factory) -> Result<(), ()>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.starts.spawn(work).map_err(|_| ())
    }

    pub(super) async fn run_start<Factory, Task, Output>(&self, work: Factory) -> Result<Output, ()>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future<Output = Output> + Send + 'static,
        Output: Send + 'static,
    {
        let admission = self.starts.try_admit().map_err(|_| ())?;
        let (sender, receiver) = tokio::sync::oneshot::channel();
        admission
            .spawn(move |cancel| async move {
                let _ = sender.send(work(cancel).await);
            })
            .map_err(|_| ())?;
        receiver.await.map_err(|_| ())
    }

    pub(super) fn try_admit_server(
        &self,
    ) -> Result<ServiceWorkAdmission<SERVER_PROCESSES>, ServiceWorkAdmissionError> {
        self.server.try_admit()
    }

    pub(super) fn begin_closing(&self) {
        self.starts.begin_closing();
        self.server.begin_closing();
    }

    pub(super) async fn stop_and_wait(&self, deadline: Instant) -> bool {
        let (starts, server) = tokio::join!(
            self.starts.stop_and_wait(deadline),
            self.server.stop_and_wait(deadline),
        );
        starts && server
    }
}
