impl OllamaManager {
    pub async fn update(
        &self,
        request: super::update::UpdateRequest,
    ) -> Result<super::update::UpdateOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Updating).await?;
        let original_bundle = guard.previous_bundle();
        self.run_admitted_update(request, guard, original_bundle).await
    }

    pub async fn update_from_release(
        &self,
        mut request: super::update::UpdateRequest,
    ) -> Result<super::update::UpdateOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Updating).await?;
        let original_bundle = guard.previous_bundle();
        request.progress = Some(
            self.progress_reporter_for_generation(guard.generation(), request.progress.take()),
        );
        self.set_operation_cancellation(request.cancellation.clone());

        let archive_names = super::release_source::archive_names_for_platform();
        let manifest = tokio::select! {
            biased;
            _ = request.cancellation.cancelled() => Err(OllamaErrorCode::OllamaOperationCancelled),
            result = super::release_source::fetch_manifest(request.version.clone(), &archive_names) => result,
        };
        let manifest = match manifest {
            Ok(manifest) => manifest,
            Err(error) => {
                // No transaction exists before the manifest arrives, so recovery
                // would only delay cancellation without anything to reconcile.
                self.clear_operation_cancellation();
                guard.succeed(original_bundle);
                return Err(error);
            }
        };
        request.manifest = Some(manifest);
        self.run_prepared_update(request, guard, original_bundle).await
    }

    async fn run_admitted_update(
        &self,
        mut request: super::update::UpdateRequest,
        guard: OllamaOperationGuard<'_>,
        original_bundle: super::types::BundleState,
    ) -> Result<super::update::UpdateOutcome, OllamaErrorCode> {
        request.progress = Some(
            self.progress_reporter_for_generation(guard.generation(), request.progress.take()),
        );
        self.set_operation_cancellation(request.cancellation.clone());
        self.run_prepared_update(request, guard, original_bundle).await
    }

    async fn run_prepared_update(
        &self,
        mut request: super::update::UpdateRequest,
        guard: OllamaOperationGuard<'_>,
        original_bundle: super::types::BundleState,
    ) -> Result<super::update::UpdateOutcome, OllamaErrorCode> {
        let recovery_paths = request.paths.clone();
        request.sidecar = self.update_sidecar(request.deadline);
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
                match super::cancel_cleanup::cleanup(recovery_paths.clone()).await {
                    Ok(()) => {
                        guard.succeed(original_bundle);
                        self.record_last_error(OllamaErrorCode::OllamaOperationCancelled);
                    }
                    Err(cleanup_error) => {
                        ::log::warn!(
                            "[ollama] cancellation cleanup deferred code={}",
                            cleanup_error.as_str()
                        );
                        drop(guard);
                        self.reconcile_after_operation_error(
                            recovery_paths,
                            OllamaErrorCode::OllamaOperationCancelled,
                        )
                        .await;
                    }
                }
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
