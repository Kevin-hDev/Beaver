// Le gestionnaire pose l'autorité avant l'adoption des consommateurs des tâches suivantes.
#![allow(dead_code)]
use super::constants::OLLAMA_WORK_CAPACITY;
use super::error::OllamaErrorCode;
use super::progress::{OllamaProgressReporter, OllamaProgressUpdate};
use super::retry::OllamaRecoveryRetry;
use super::startup::OllamaStartupBarrier;
use super::types::{OllamaRuntimeStatus, OperationState};
use crate::app_exit::{AppEmergencyPublisher, AppWorkSupervisor};
#[cfg(test)]
use crate::services::work_registry::ServiceWorkDiagnostics;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkSupervisor,
};
use std::sync::{Arc, Mutex};
#[cfg(test)]
use tokio::sync::Notify;
use tokio::sync::{Mutex as AsyncMutex, MutexGuard};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct OllamaManager(Arc<OllamaManagerInner>);

struct OllamaManagerInner {
    work: ServiceWorkSupervisor<OLLAMA_WORK_CAPACITY>,
    operation_lock: AsyncMutex<()>,
    state: Mutex<OllamaManagerState>,
    owned_process: Mutex<Option<super::process::OwnedOllamaProcess>>,
    active_cancellation: Mutex<Option<CancellationToken>>,
    emergency: Option<AppEmergencyPublisher>,
    startup: OllamaStartupBarrier,
    retry: OllamaRecoveryRetry,
}

struct OllamaManagerState {
    closing: bool,
    generation: u64,
    status: OllamaRuntimeStatus,
}

pub(crate) struct OllamaOperationGuard<'a> {
    manager: &'a OllamaManagerInner,
    admission: ServiceWorkAdmission<OLLAMA_WORK_CAPACITY>,
    #[allow(dead_code)]
    operation_lock: MutexGuard<'a, ()>,
    generation: u64,
}

impl OllamaManager {
    pub fn new(app_work: AppWorkSupervisor) -> Self {
        Self::new_inner(app_work, None)
    }

    pub async fn status(&self) -> OllamaRuntimeStatus {
        self.inner().lock_state().status.clone()
    }

    pub(crate) async fn begin_operation(
        &self,
        operation: OperationState,
    ) -> Result<OllamaOperationGuard<'_>, OllamaErrorCode> {
        let admission = self.inner().work.try_admit().map_err(map_admission_error)?;
        self.begin_operation_after_admission(admission, operation)
            .await
    }

    async fn begin_operation_after_admission(
        &self,
        admission: ServiceWorkAdmission<OLLAMA_WORK_CAPACITY>,
        operation: OperationState,
    ) -> Result<OllamaOperationGuard<'_>, OllamaErrorCode> {
        let operation_lock = self.inner().operation_lock.lock().await;
        let generation = {
            let mut state = self.inner().lock_state();
            if state.closing || admission.cancellation().is_cancelled() {
                return Err(OllamaErrorCode::OllamaClosing);
            }
            let generation = state
                .generation
                .checked_add(1)
                .ok_or(OllamaErrorCode::OllamaInternal)?;
            state.generation = generation;
            state.status.operation = operation;
            if matches!(
                operation,
                OperationState::Installing | OperationState::Updating
            ) {
                state.status.bundle = super::types::BundleState::TransactionPending;
                state.status.progress = Some(super::types::OllamaProgressStage::Preparing);
            } else {
                state.status.progress = matches!(operation, OperationState::Recovering)
                    .then_some(super::types::OllamaProgressStage::Recovering);
            }
            state.status.last_error = None;
            generation
        };
        Ok(OllamaOperationGuard {
            manager: self.inner(),
            admission,
            operation_lock,
            generation,
        })
    }
    #[cfg(test)]
    pub(crate) async fn begin_operation_paused_for_test(
        &self,
        operation: OperationState,
        admitted: Arc<Notify>,
        resume: Arc<Notify>,
    ) -> Result<OllamaOperationGuard<'_>, OllamaErrorCode> {
        let admission = self.inner().work.try_admit().map_err(map_admission_error)?;
        admitted.notify_one();
        resume.notified().await;
        self.begin_operation_after_admission(admission, operation)
            .await
    }

    fn inner(&self) -> &OllamaManagerInner {
        &self.0
    }

    fn release_generation(&self, generation: u64, cancelled: bool) {
        self.inner().release_generation(generation, cancelled);
    }

    pub(crate) fn progress_reporter_for_generation(
        &self,
        generation: u64,
        forward: Option<OllamaProgressReporter>,
    ) -> OllamaProgressReporter {
        let manager = self.clone();
        Arc::new(move |update| {
            manager.publish_progress_for_generation(generation, update);
            if let Some(reporter) = forward.as_ref() {
                reporter(update);
            }
        })
    }

    fn publish_progress_for_generation(&self, generation: u64, update: OllamaProgressUpdate) {
        let mut state = self.inner().lock_state();
        if state.generation == generation {
            state.status.progress = Some(update.stage);
        }
    }

    #[cfg(test)]
    pub(crate) fn work_diagnostics_for_test(&self) -> ServiceWorkDiagnostics {
        self.inner().work.diagnostics()
    }

    #[cfg(test)]
    pub(crate) fn generation_for_test(&self) -> u64 {
        self.inner().lock_state().generation
    }

    #[cfg(test)]
    pub(crate) fn set_generation_for_test(&self, generation: u64) {
        self.inner().lock_state().generation = generation;
    }

    #[cfg(test)]
    pub(crate) fn supersede_generation_for_test(&self, operation: OperationState) {
        let mut state = self.inner().lock_state();
        state.generation = state.generation.checked_add(1).expect("test generation");
        state.status.operation = operation;
    }

    #[cfg(test)]
    pub(crate) fn release_generation_for_test(&self, generation: u64) {
        self.release_generation(generation, false);
    }
}

impl OllamaManagerInner {
    fn mark_closing(&self) {
        let mut state = self.lock_state();
        state.closing = true;
        if !matches!(state.status.operation, OperationState::Idle) {
            state.status.operation = OperationState::Cancelling;
            state.status.progress = None;
        }
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, OllamaManagerState> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => {
                self.work.begin_closing();
                let mut state = poisoned.into_inner();
                state.closing = true;
                state.status.operation = OperationState::Cancelling;
                state.status.progress = None;
                state.status.last_error = Some(OllamaErrorCode::OllamaInternal);
                state
            }
        }
    }

    fn release_generation(&self, generation: u64, cancelled: bool) {
        let mut state = self.lock_state();
        if state.generation != generation {
            return;
        }
        state.status.operation = if state.closing || cancelled {
            OperationState::Cancelling
        } else {
            OperationState::Idle
        };
        state.status.progress = None;
    }
}

fn map_admission_error(error: ServiceWorkAdmissionError) -> OllamaErrorCode {
    match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => {
            OllamaErrorCode::OllamaClosing
        }
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            OllamaErrorCode::OllamaOperationInProgress
        }
    }
}
include!("manager_operation_guard.rs");
include!("manager_update.rs");
include!("manager_startup.rs");
