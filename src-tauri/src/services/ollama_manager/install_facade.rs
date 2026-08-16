#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::install::{self, InstallOutcome, InstallRequest};
use super::manager::OllamaManager;
use super::types::OperationState;

impl OllamaManager {
    pub async fn install(
        &self,
        mut request: InstallRequest,
    ) -> Result<InstallOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Installing).await?;
        request.progress = Some(
            self.progress_reporter_for_generation(guard.generation(), request.progress.take()),
        );
        let recovery_paths = request.paths.clone();
        self.set_operation_cancellation(request.cancellation.clone());
        let result = install::install(request).await;
        self.clear_operation_cancellation();
        match &result {
            Ok(InstallOutcome::Installed { .. }) => {
                guard.succeed(super::types::BundleState::Ready);
            }
            Ok(InstallOutcome::Preparing) => drop(guard),
            Err(error) => {
                drop(guard);
                self.reconcile_after_operation_error(recovery_paths, *error)
                    .await;
            }
        }
        result
    }
}
