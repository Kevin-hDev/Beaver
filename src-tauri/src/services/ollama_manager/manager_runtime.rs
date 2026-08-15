use super::types::{
    BundleState, CancelOutcome, DaemonState, OllamaCliArgs, OllamaCliOutput, OllamaEndpoint,
    OllamaStartOutcome,
};
use std::time::{Duration, Instant};

impl OllamaManager {
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
        let mut state = self.inner().lock_state();
        if matches!(state.status.operation, OperationState::Idle) {
            return CancelOutcome::AlreadyIdle;
        }
        state.status.operation = OperationState::Cancelling;
        state.status.last_error = Some(OllamaErrorCode::OllamaOperationCancelled);
        CancelOutcome::Cancelled
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

    pub async fn stop_and_wait(&self, deadline: Instant) -> Result<(), OllamaErrorCode> {
        self.stop_impl(deadline).await
    }

    pub async fn run_cli(
        &self,
        args: OllamaCliArgs,
    ) -> Result<OllamaCliOutput, OllamaErrorCode> {
        self.run_cli_impl(args).await
    }

    fn publish_bundle_ready(&self) {
        self.inner().lock_state().status.bundle = BundleState::Ready;
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
            emergency,
            startup: super::startup::OllamaStartupBarrier::new(),
            retry: super::retry::OllamaRecoveryRetry::new(),
        }))
    }
}

include!("manager_process.rs");
