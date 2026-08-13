use super::types::{MAX_IN_FLIGHT_REQUESTS, MAX_PENDING_REQUESTS};
use crate::app_exit::AppWorkSupervisor;
#[cfg(test)]
use crate::services::work_registry::ServiceWorkDiagnostics;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkSupervisor,
};
use std::future::Future;
use std::time::Instant;

const EXTENSION_HOST_READERS: usize = 1;
pub(super) const MAX_EXTENSION_OPERATIONS: usize = MAX_PENDING_REQUESTS;
pub(super) const MAX_EXTENSION_CORE_CALLS: usize = MAX_IN_FLIGHT_REQUESTS;

type ExtensionReaderWork = ServiceWorkSupervisor<EXTENSION_HOST_READERS>;
type ExtensionOperationWork = ServiceWorkSupervisor<MAX_EXTENSION_OPERATIONS>;
type ExtensionCoreCallWork = ServiceWorkSupervisor<MAX_EXTENSION_CORE_CALLS>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExtensionWorkAdmissionError {
    ShuttingDown,
    Busy,
}

impl ExtensionWorkAdmissionError {
    pub(super) fn public_code(self) -> &'static str {
        match self {
            Self::ShuttingDown => super::error_codes::HOST_UNAVAILABLE,
            Self::Busy => super::error_codes::HOST_BUSY,
        }
    }

    pub(super) fn operation_failure(self) -> super::OperationFailure {
        match self {
            Self::ShuttingDown => super::OperationFailure::HostUnavailable,
            Self::Busy => super::OperationFailure::HostBusy,
        }
    }
}

#[derive(Clone)]
pub(super) struct ExtensionWorkServices {
    readers: ExtensionReaderWork,
    operations: ExtensionOperationWork,
    core_calls: ExtensionCoreCallWork,
}

impl ExtensionWorkServices {
    pub(super) fn new(app: AppWorkSupervisor) -> Self {
        Self {
            readers: ExtensionReaderWork::new(app.clone()),
            operations: ExtensionOperationWork::new(app.clone()),
            core_calls: ExtensionCoreCallWork::new(app),
        }
    }

    pub(super) fn try_admit_operation(
        &self,
    ) -> Result<ServiceWorkAdmission<MAX_EXTENSION_OPERATIONS>, ExtensionWorkAdmissionError> {
        self.operations.try_admit().map_err(map_admission_error)
    }

    pub(super) fn try_admit_reader(
        &self,
    ) -> Result<ServiceWorkAdmission<EXTENSION_HOST_READERS>, ExtensionWorkAdmissionError> {
        self.readers.try_admit().map_err(map_admission_error)
    }

    #[cfg(test)]
    pub(super) fn try_admit_core_call(
        &self,
    ) -> Result<ServiceWorkAdmission<MAX_EXTENSION_CORE_CALLS>, ExtensionWorkAdmissionError> {
        self.core_calls.try_admit().map_err(map_admission_error)
    }

    pub(super) fn spawn_operation<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ExtensionWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.operations.spawn(work).map_err(map_admission_error)
    }

    pub(super) async fn run_operation<Factory, Task, Output>(
        &self,
        work: Factory,
    ) -> Result<Output, ExtensionWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future<Output = Output> + Send + 'static,
        Output: Send + 'static,
    {
        let admission = self.try_admit_operation()?;
        let (sender, receiver) = tokio::sync::oneshot::channel();
        admission
            .spawn(move |cancel| async move {
                let output = work(cancel).await;
                let _ = sender.send(output);
            })
            .map_err(map_admission_error)?;
        receiver
            .await
            .map_err(|_| ExtensionWorkAdmissionError::ShuttingDown)
    }

    pub(super) fn spawn_core_call<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ExtensionWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.core_calls.spawn(work).map_err(map_admission_error)
    }

    pub(super) fn begin_closing(&self) {
        // Les trois producteurs ferment avant que l'hôte ne soit tué : aucun
        // lecteur, appel ou redémarrage ne peut franchir cette frontière.
        self.readers.begin_closing();
        self.operations.begin_closing();
        self.core_calls.begin_closing();
    }

    pub(super) fn is_open(&self) -> bool {
        self.operations.phase() == crate::services::work_registry::ServiceWorkPhase::Open
    }

    pub(super) async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.begin_closing();
        let (readers, operations, core_calls) = tokio::join!(
            self.readers.stop_and_wait(deadline),
            self.operations.stop_and_wait(deadline),
            self.core_calls.stop_and_wait(deadline),
        );
        readers && operations && core_calls
    }

    #[cfg(test)]
    pub(super) fn operation_diagnostics(&self) -> ServiceWorkDiagnostics {
        self.operations.diagnostics()
    }

    #[cfg(test)]
    pub(super) fn core_call_diagnostics(&self) -> ServiceWorkDiagnostics {
        self.core_calls.diagnostics()
    }

    #[cfg(test)]
    pub(super) fn reader_phase(&self) -> crate::services::work_registry::ServiceWorkPhase {
        self.readers.phase()
    }

    #[cfg(test)]
    pub(super) fn operation_phase(&self) -> crate::services::work_registry::ServiceWorkPhase {
        self.operations.phase()
    }

    #[cfg(test)]
    pub(super) fn core_call_phase(&self) -> crate::services::work_registry::ServiceWorkPhase {
        self.core_calls.phase()
    }
}

fn map_admission_error(error: ServiceWorkAdmissionError) -> ExtensionWorkAdmissionError {
    match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => {
            ExtensionWorkAdmissionError::ShuttingDown
        }
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            ExtensionWorkAdmissionError::Busy
        }
    }
}

#[cfg(test)]
pub(super) fn open_cancellation_for_test() -> ServiceWorkCancellation {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work = ExtensionWorkServices::new(coordinator.work_supervisor());
    work.try_admit_operation().unwrap().cancellation()
}
