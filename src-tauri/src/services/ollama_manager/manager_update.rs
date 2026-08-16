impl OllamaManager {
    pub async fn update(
        &self,
        mut request: super::update::UpdateRequest,
    ) -> Result<super::update::UpdateOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Updating).await?;
        request.progress = Some(self.progress_reporter_for_generation(
            guard.generation(),
            request.progress.take(),
        ));
        let recovery_paths = request.paths.clone();
        let deadline = request
            .deadline
            .unwrap_or_else(|| std::time::Instant::now() + std::time::Duration::from_secs(15));
        request.sidecar = self.update_sidecar(deadline);
        self.set_operation_cancellation(request.cancellation.clone());
        let result = super::update::run(request).await;
        self.clear_operation_cancellation();
        match &result {
            Ok(super::update::UpdateOutcome::Updated { .. })
            | Ok(super::update::UpdateOutcome::AlreadyCurrent) => {
                guard.succeed(super::types::BundleState::Ready);
            }
            Ok(super::update::UpdateOutcome::CleanupPending { code })
            | Ok(super::update::UpdateOutcome::Deferred { code }) => {
                drop(guard);
                if matches!(
                    self.run_startup_recovery_at(recovery_paths).await,
                    super::startup::StartupBarrierState::Ready
                ) {
                    self.record_last_error(*code);
                }
            }
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
