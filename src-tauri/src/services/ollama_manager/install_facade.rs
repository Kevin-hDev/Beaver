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
        self.set_operation_cancellation(request.cancellation.clone());
        let result = install::install(request).await;
        self.clear_operation_cancellation();
        if let Err(error) = result {
            guard.fail(error);
        } else {
            drop(guard);
        }
        result
    }
}
