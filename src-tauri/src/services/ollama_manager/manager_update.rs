impl OllamaManager {
    pub async fn update(
        &self,
        mut request: super::update::UpdateRequest,
    ) -> Result<super::update::UpdateOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Updating).await?;
        request.progress = Some(
            self.progress_reporter_for_generation(guard.generation(), request.progress.take()),
        );
        let recovery_paths = request.paths.clone();
        request.sidecar = self.update_sidecar(request.deadline);
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
                ::log::warn!("[ollama] update incomplete code={}", code.as_str());
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
                self.reconcile_after_operation_error(
                    recovery_paths,
                    OllamaErrorCode::OllamaOperationCancelled,
                )
                .await;
            }
            Err(error) => {
                ::log::error!("[ollama] update failed code={}", error.as_str());
                drop(guard);
                self.reconcile_after_operation_error(recovery_paths, *error)
                    .await;
            }
        }
        result
    }
}
