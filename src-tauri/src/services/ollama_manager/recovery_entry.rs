#![allow(dead_code)]

use super::durable_fs::platform_fs;
use super::error::OllamaErrorCode;
use super::manager::OllamaManager;
use super::recovery::{RecoveryExecutor, RecoveryOutcome, RecoveryReason};
use super::types::OperationState;
use crate::services::paths::{data_dir, ollama_paths};
use std::sync::Arc;

pub(crate) async fn recover_platform(
    reason: RecoveryReason,
) -> Result<RecoveryOutcome, OllamaErrorCode> {
    let paths = ollama_paths(&data_dir());
    RecoveryExecutor::new(Arc::new(platform_fs()), Arc::new(()), paths)
        .recover(reason)
        .await
}

impl OllamaManager {
    pub async fn recover(
        &self,
        reason: RecoveryReason,
    ) -> Result<RecoveryOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Recovering).await?;
        let result = recover_platform(reason).await;
        match &result {
            Err(code) => guard.fail(*code),
            Ok(RecoveryOutcome::Deferred { code }) => guard.fail(*code),
            Ok(_) => drop(guard),
        }
        result
    }
}
