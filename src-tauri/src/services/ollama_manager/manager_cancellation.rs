use super::types::CancelOutcome;

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

    pub(super) fn cancel_active_operation(&self) {
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
                let active = self.inner().active_cancellation.lock();
                return match active {
                    Ok(active) => match active.as_ref() {
                        Some(cancellation) => {
                            cancellation.cancel();
                            CancelOutcome::Cancelled
                        }
                        None => CancelOutcome::AlreadyIdle,
                    },
                    Err(_) => CancelOutcome::Cancelled,
                };
            }
            state.status.operation = OperationState::Cancelling;
            state.status.last_error = Some(OllamaErrorCode::OllamaOperationCancelled);
            CancelOutcome::Cancelled
        };
        self.cancel_active_operation();
        outcome
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
