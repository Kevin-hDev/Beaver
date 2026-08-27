use super::types::CancelOutcome;

const CANCELLATION_ADMISSION_POLL: Duration = Duration::from_millis(25);

impl OllamaManager {
    fn set_active_cancellation(&self, cancellation: tokio_util::sync::CancellationToken) {
        // The state lock orders admission and token registration so a click can
        // never fall into the small gap between those two operations.
        let state = self.inner().lock_state();
        if matches!(state.status.operation, OperationState::Cancelling) {
            cancellation.cancel();
        }
        if let Ok(mut active) = self.inner().active_cancellation.lock() {
            *active = Some(cancellation);
        }
    }

    fn clear_active_cancellation(&self) {
        if let Ok(mut active) = self.inner().active_cancellation.lock() {
            *active = None;
        }
    }

    fn cancel_active_operation(&self) {
        if let Ok(active) = self.inner().active_cancellation.lock() {
            if let Some(cancellation) = active.as_ref() {
                cancellation.cancel();
            }
        }
    }

    pub async fn cancel_operation(&self) -> CancelOutcome {
        if self.is_closing() {
            return CancelOutcome::RejectedDuringShutdown;
        }
        let outcome = {
            let mut state = self.inner().lock_state();
            if matches!(state.status.operation, OperationState::Idle) {
                return CancelOutcome::AlreadyIdle;
            }
            state.status.operation = OperationState::Cancelling;
            state.status.last_error = Some(OllamaErrorCode::OllamaOperationCancelled);
            CancelOutcome::Cancelled
        };
        self.cancel_active_operation();
        outcome
    }

    pub async fn cancel_operation_when_admitted(&self, max_wait: Duration) -> CancelOutcome {
        let deadline = Instant::now() + max_wait;
        loop {
            let outcome = self.cancel_operation().await;
            if !matches!(outcome, CancelOutcome::AlreadyIdle) || Instant::now() >= deadline {
                return outcome;
            }
            tokio::time::sleep(CANCELLATION_ADMISSION_POLL).await;
        }
    }

    pub(crate) fn set_operation_cancellation(
        &self,
        cancellation: tokio_util::sync::CancellationToken,
    ) {
        self.set_active_cancellation(cancellation);
    }

    pub(crate) fn clear_operation_cancellation(&self) {
        self.clear_active_cancellation();
    }
}
