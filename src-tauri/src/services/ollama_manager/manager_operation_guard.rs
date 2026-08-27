impl<'a> OllamaOperationGuard<'a> {
    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn previous_bundle(&self) -> super::types::BundleState {
        self.previous_bundle.clone()
    }

    #[cfg(test)]
    pub(crate) fn generation_for_test(&self) -> u64 {
        self.generation()
    }

    pub(super) fn succeed(self, bundle: super::types::BundleState) {
        let mut state = self.manager.lock_state();
        if state.generation == self.generation {
            state.status.bundle = bundle;
            state.status.progress = None;
            state.status.last_error = None;
        }
    }

    pub(super) fn fail(self, error: OllamaErrorCode) {
        let mut state = self.manager.lock_state();
        if state.generation == self.generation {
            state.status.bundle = super::types::BundleState::RecoveryRequired;
            state.status.last_error = Some(error);
        }
    }

    #[cfg(test)]
    pub(crate) fn fail_for_test(self, error: OllamaErrorCode) {
        self.fail(error);
    }
}

impl OllamaManager {
    pub(super) fn record_last_error(&self, error: OllamaErrorCode) {
        self.inner().lock_state().status.last_error = Some(error);
    }
}

impl Drop for OllamaOperationGuard<'_> {
    fn drop(&mut self) {
        // Ce champ est conservé pour maintenir le verrou pendant toute la transaction.
        let _ = &self.operation_lock;
        self.manager.release_generation(
            self.generation,
            self.admission.cancellation().is_cancelled(),
        );
    }
}
