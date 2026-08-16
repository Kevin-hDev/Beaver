#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::install::{self, InstallOutcome, InstallRequest};
use super::manager::OllamaManager;
use super::types::OperationState;

impl OllamaManager {
    pub async fn install(
        &self,
        request: InstallRequest,
    ) -> Result<InstallOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Installing).await?;
        let recovery_paths = request.paths.clone();
        self.set_operation_cancellation(request.cancellation.clone());
        let result = install::install(request).await;
        self.clear_operation_cancellation();
        match &result {
            Ok(InstallOutcome::Installed { .. }) => {
                guard.succeed(super::types::BundleState::Ready);
            }
            Ok(InstallOutcome::Preparing) => drop(guard),
            Err(OllamaErrorCode::OllamaOperationCancelled) => {
                drop(guard);
                if matches!(
                    self.run_startup_recovery_at(recovery_paths).await,
                    super::startup::StartupBarrierState::Ready
                ) {
                    self.record_last_error(OllamaErrorCode::OllamaOperationCancelled);
                }
            }
            Err(error) => guard.fail(*error),
        }
        result
    }
}
