use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkSupervisor,
};
use std::future::Future;
use std::time::Instant;

// Les opérations Forecast viennent des mêmes flux UI et agent que les autres outils ;
// réutiliser leur borne évite une seconde capacité concurrente pour le même travail.
const FORECAST_OPERATIONS: usize =
    crate::services::agent_local::agent_work_supervision::MAX_ACTIVE_AGENT_STREAMS;
const SIDECAR_PROCESSES: usize = 1;
const IDLE_WORKERS: usize = 1;

pub(super) type SidecarAdmission = ServiceWorkAdmission<SIDECAR_PROCESSES>;
type OperationWork = ServiceWorkSupervisor<FORECAST_OPERATIONS>;
type SidecarWork = ServiceWorkSupervisor<SIDECAR_PROCESSES>;
type IdleWork = ServiceWorkSupervisor<IDLE_WORKERS>;

#[derive(Clone)]
pub(super) struct ForecastWorkServices {
    operations: OperationWork,
    sidecar: SidecarWork,
    idle: IdleWork,
}

impl ForecastWorkServices {
    pub(super) fn new(app: AppWorkSupervisor) -> Self {
        Self {
            operations: OperationWork::new(app.clone()),
            sidecar: SidecarWork::new(app.clone()),
            idle: IdleWork::new(app),
        }
    }

    pub(super) async fn run_operation<Factory, Task, Output>(
        &self,
        work: Factory,
    ) -> Result<Output, String>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future<Output = Result<Output, String>> + Send + 'static,
        Output: Send + 'static,
    {
        let admission = self
            .operations
            .try_admit()
            .map_err(public_admission_error)?;
        let (sender, receiver) = tokio::sync::oneshot::channel();
        admission
            .spawn(move |cancel| async move {
                let _ = sender.send(work(cancel).await);
            })
            .map_err(public_admission_error)?;
        receiver
            .await
            .map_err(|_| "forecast-operation-interrupted".to_string())?
    }

    #[cfg(test)]
    pub(super) fn try_admit_operation(
        &self,
    ) -> Result<ServiceWorkAdmission<FORECAST_OPERATIONS>, ServiceWorkAdmissionError> {
        self.operations.try_admit()
    }

    pub(super) fn try_admit_sidecar(&self) -> Result<SidecarAdmission, ServiceWorkAdmissionError> {
        self.sidecar.try_admit()
    }

    pub(super) fn spawn_idle<Factory, Task>(&self, work: Factory) -> Result<(), ()>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.idle.spawn(work).map_err(|_| ())
    }

    pub(super) fn begin_closing(&self) {
        self.operations.begin_closing();
        self.sidecar.begin_closing();
        self.idle.begin_closing();
    }

    pub(super) async fn stop_and_wait(&self, deadline: Instant) -> bool {
        let (operations, sidecar, idle) = tokio::join!(
            self.operations.stop_and_wait(deadline),
            self.sidecar.stop_and_wait(deadline),
            self.idle.stop_and_wait(deadline),
        );
        operations && sidecar && idle
    }

    #[cfg(test)]
    pub(super) fn idle_counts_for_test(&self) -> (usize, usize) {
        let diagnostics = self.idle.diagnostics();
        (diagnostics.active, diagnostics.high_water)
    }
}

fn public_admission_error(error: ServiceWorkAdmissionError) -> String {
    error.public_code().to_string()
}
