impl OllamaManager {
    pub async fn update(
        &self,
        request: super::update::UpdateRequest,
    ) -> Result<super::update::UpdateOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Updating).await?;
        let result = super::update::run(request).await;
        if let Err(error) = result {
            guard.fail(error);
        } else {
            drop(guard);
        }
        result
    }
}
