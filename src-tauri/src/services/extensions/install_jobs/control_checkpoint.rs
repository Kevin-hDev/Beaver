use super::{checkpoint::InstallCheckpoint, InstallControl, InstallInterruption, InstallPhase};

impl InstallControl {
    pub(in crate::services::extensions) fn saved(
        &self,
    ) -> Result<Option<InstallCheckpoint>, InstallInterruption> {
        let state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
        Ok(state.jobs[state
            .index(&self.id)
            .map_err(|_| InstallInterruption::Failed)?]
        .checkpoint
        .clone())
    }
    pub(in crate::services::extensions) fn save(
        &self,
        mut checkpoint: InstallCheckpoint,
    ) -> Result<(), InstallInterruption> {
        let mut state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
        let index = state
            .index(&self.id)
            .map_err(|_| InstallInterruption::Failed)?;
        // The store owns consent; a phase's older checkpoint cannot overwrite it.
        if let Some(current) = &state.jobs[index].checkpoint {
            if current.token == checkpoint.token {
                checkpoint.allowance = current.allowance.clone();
                checkpoint.dependency_lock = current.dependency_lock.clone();
                checkpoint.resolved_source = current
                    .resolved_source
                    .clone()
                    .or(checkpoint.resolved_source);
            }
        }
        if let Some(record) = &checkpoint.record {
            state.jobs[index].view.display_name = record.manifest.name.clone();
        }
        state.jobs[index].checkpoint = Some(checkpoint);
        if self.store.persist(&state).is_err() {
            state.durable_error = true;
            self.cancel.cancel();
            return Err(InstallInterruption::Failed);
        }
        Ok(())
    }
    pub(super) fn publish(
        &self,
        operation: impl FnOnce() -> Result<String, InstallInterruption>,
    ) -> Result<String, InstallInterruption> {
        self.checkpoint(InstallPhase::Publishing)?;
        let mut state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
        if self.is_cancelled() || state.durable_error {
            return Err(InstallInterruption::Cancelled);
        }
        let index = state
            .index(&self.id)
            .map_err(|_| InstallInterruption::Failed)?;
        // Cancellation and publication share this short boundary. No await occurs here.
        state.jobs[index].view.can_cancel = false;
        self.store
            .persist(&state)
            .map_err(|_| InstallInterruption::Failed)?;
        let result = operation();
        if let Ok(id) = &result {
            state.jobs[index].view.status = super::InstallStatus::Completed;
            state.jobs[index].view.extension_id = Some(id.clone());
            self.store.changed(&mut state);
            if let Some(app) = &self.store.app {
                use tauri::Emitter;
                if app
                    .emit(super::super::runtime_lifecycle::CHANGED_EVENT, ())
                    .is_err()
                {
                    log::warn!("extension registry notification delivery failed");
                }
            }
        }
        result
    }
}
