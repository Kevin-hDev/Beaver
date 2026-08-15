impl OllamaManager {
    pub async fn update(
        &self,
        mut request: super::update::UpdateRequest,
    ) -> Result<super::update::UpdateOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Updating).await?;
        let deadline = request
            .deadline
            .unwrap_or_else(|| std::time::Instant::now() + std::time::Duration::from_secs(15));
        request.sidecar = self.update_sidecar(deadline);
        self.set_operation_cancellation(request.cancellation.clone());
        let result = super::update::run(request).await;
        self.clear_operation_cancellation();
        if let Err(error) = result {
            guard.fail(error);
        } else {
            drop(guard);
        }
        result
    }
}
