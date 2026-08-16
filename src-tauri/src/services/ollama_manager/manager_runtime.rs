use super::types::{
    BundleState, CancelOutcome, DaemonState, OllamaCliArgs, OllamaCliOutput, OllamaEndpoint,
    OllamaStartOutcome,
};
use super::update::{OwnedSidecarController, UpdateSidecar};
use std::time::{Duration, Instant};

impl OllamaManager {
    fn set_active_cancellation(&self, cancellation: tokio_util::sync::CancellationToken) {
        if let Ok(mut active) = self.inner().active_cancellation.lock() {
            *active = Some(cancellation);
        }
    }

    fn clear_active_cancellation(&self) {
        if let Ok(mut active) = self.inner().active_cancellation.lock() {
            *active = None;
        }
    }

    fn cancel_active_operation(&self) {
        if let Ok(active) = self.inner().active_cancellation.lock() {
            if let Some(cancellation) = active.as_ref() {
                cancellation.cancel();
            }
        }
    }

    pub async fn start(&self) -> OllamaStartOutcome {
        self.start_impl().await
    }

    pub async fn restart(&self) -> OllamaStartOutcome {
        if self.is_closing() {
            return OllamaStartOutcome::RejectedDuringShutdown;
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        if self.stop_and_wait(deadline).await.is_err() {
            return OllamaStartOutcome::Failed {
                code: OllamaErrorCode::OllamaStopFailed,
            };
        }
        self.start().await
    }

    pub async fn cancel_operation(&self) -> CancelOutcome {
        if self.is_closing() {
            return CancelOutcome::RejectedDuringShutdown;
        }
        let outcome = {
            let mut state = self.inner().lock_state();
            if matches!(state.status.operation, OperationState::Idle) {
                return CancelOutcome::AlreadyIdle;
            }
            state.status.operation = OperationState::Cancelling;
            state.status.last_error = Some(OllamaErrorCode::OllamaOperationCancelled);
            CancelOutcome::Cancelled
        };
        self.cancel_active_operation();
        outcome
    }

    pub async fn installed_version(&self) -> Option<super::fingerprint::OllamaVersion> {
        let paths = crate::services::paths::ollama_paths(&crate::services::paths::data_dir());
        super::bundle_receipt::read_receipt(
            &super::durable_fs::platform_fs(),
            &crate::services::paths::bundle_receipt_path(&paths.active),
        )
        .ok()
        .flatten()
        .map(|receipt| receipt.fingerprint.version)
    }

    pub(crate) fn update_sidecar(&self, deadline: Instant) -> UpdateSidecar {
        match self.inner().lock_state().status.daemon {
            DaemonState::Owned { .. } => {
                UpdateSidecar::Owned(std::sync::Arc::new(ManagerSidecarController {
                    manager: self.clone(),
                    deadline,
                }))
            }
            DaemonState::External { .. } => UpdateSidecar::External,
            DaemonState::Unavailable => UpdateSidecar::Absent,
        }
    }

    pub(crate) fn set_operation_cancellation(
        &self,
        cancellation: tokio_util::sync::CancellationToken,
    ) {
        self.set_active_cancellation(cancellation);
    }

    pub(crate) fn clear_operation_cancellation(&self) {
        self.clear_active_cancellation();
    }

    pub(crate) fn manager_sidecar_stop(&self) -> Result<(), super::error::OllamaErrorCode> {
        let mut process = self
            .inner()
            .owned_process
            .lock()
            .map_err(|_| super::error::OllamaErrorCode::OllamaStopFailed)?;
        process
            .as_mut()
            .ok_or(super::error::OllamaErrorCode::OllamaStopFailed)?
            .terminate()
            .map_err(map_process_error)
    }

    pub(crate) fn manager_sidecar_reap(
        &self,
        deadline: Instant,
    ) -> Result<(), super::error::OllamaErrorCode> {
        let mut process = self
            .inner()
            .owned_process
            .lock()
            .map_err(|_| super::error::OllamaErrorCode::OllamaStopFailed)?;
        process
            .as_mut()
            .ok_or(super::error::OllamaErrorCode::OllamaStopFailed)?
            .reap(deadline)
            .map_err(map_process_error)?;
        process.take();
        self.publish_daemon(DaemonState::Unavailable);
        Ok(())
    }

    pub async fn usable_endpoint(&self) -> Result<OllamaEndpoint, OllamaErrorCode> {
        match self.status().await.daemon {
            DaemonState::Owned { endpoint } | DaemonState::External { endpoint } => Ok(endpoint),
            DaemonState::Unavailable => Err(OllamaErrorCode::OllamaUnavailable),
        }
    }

    pub async fn owned_endpoint(&self) -> Option<OllamaEndpoint> {
        match self.status().await.daemon {
            DaemonState::Owned { endpoint } => Some(endpoint),
            _ => None,
        }
    }

    pub(crate) fn publish_external_daemon(&self, endpoint: OllamaEndpoint) {
        self.publish_daemon(DaemonState::External { endpoint });
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> Result<(), OllamaErrorCode> {
        self.stop_impl(deadline).await
    }

    pub async fn run_cli(&self, args: OllamaCliArgs) -> Result<OllamaCliOutput, OllamaErrorCode> {
        self.run_cli_impl(args).await
    }

    fn publish_bundle_ready(&self) {
        self.inner().lock_state().status.bundle = BundleState::Ready;
    }
}

struct ManagerSidecarController {
    manager: OllamaManager,
    deadline: Instant,
}

impl OwnedSidecarController for ManagerSidecarController {
    fn stop(&self) -> Result<(), OllamaErrorCode> {
        self.manager.manager_sidecar_stop()
    }

    fn reap(&self) -> Result<(), OllamaErrorCode> {
        self.manager.manager_sidecar_reap(self.deadline)
    }
}

impl OllamaManager {
    pub(crate) fn with_emergency(
        app_work: crate::app_exit::AppWorkSupervisor,
        emergency: crate::app_exit::AppEmergencyPublisher,
    ) -> Self {
        Self::new_inner(app_work, Some(emergency))
    }

    fn new_inner(
        app_work: crate::app_exit::AppWorkSupervisor,
        emergency: Option<crate::app_exit::AppEmergencyPublisher>,
    ) -> Self {
        Self(std::sync::Arc::new(super::manager::OllamaManagerInner {
            work: crate::services::work_registry::ServiceWorkSupervisor::new(app_work),
            operation_lock: tokio::sync::Mutex::new(()),
            state: std::sync::Mutex::new(super::manager::OllamaManagerState {
                closing: false,
                generation: 0,
                status: super::types::OllamaRuntimeStatus::initial(),
            }),
            owned_process: std::sync::Mutex::new(None),
            active_cancellation: std::sync::Mutex::new(None),
            emergency,
            startup: super::startup::OllamaStartupBarrier::new(),
            retry: super::retry::OllamaRecoveryRetry::new(),
        }))
    }
}

include!("manager_process.rs");
