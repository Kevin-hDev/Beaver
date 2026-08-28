use super::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkKey,
    ServiceWorkPhase, ServiceWorkSupervisor, WorkRegistry,
};
use crate::app_exit::{AppWorkAdmissionError, AppWorkSupervisor};
use std::future::Future;

impl ServiceWorkCancellation {
    pub async fn cancelled(&self) {
        tokio::select! {
            _ = self.app.cancelled() => {},
            _ = self.service.cancelled() => {},
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.app.is_cancelled() || self.service.is_cancelled()
    }

    pub(crate) fn cancel(&self) {
        // Chaque admission reçoit un enfant distinct : l'annuler arrête ce
        // travail sans fermer le service et sans toucher aux futurs travaux.
        self.service.cancel();
    }
}

impl<const CAPACITY: usize> ServiceWorkAdmission<CAPACITY> {
    pub fn cancellation(&self) -> ServiceWorkCancellation {
        self.cancellation.clone()
    }

    pub async fn run<F>(self, future: F) -> F::Output
    where
        F: Future,
    {
        let guard = self;
        let output = future.await;
        drop(guard);
        output
    }

    pub fn spawn<Factory, Task>(self, work: Factory) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.spawn_with_completion(work).map(drop)
    }

    pub fn spawn_with_completion<Factory, Task>(
        self,
        work: Factory,
    ) -> Result<tokio::sync::oneshot::Receiver<()>, ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        let key = self.key.expect("new admission has a key");
        let cancellation = self.cancellation();
        let registry = self.registry.clone();
        let (completed, completion) = tokio::sync::oneshot::channel();
        let mut state = registry.lock_state();
        if state.phase != ServiceWorkPhase::Open {
            state.diagnostics.closing_refusals =
                state.diagnostics.closing_refusals.saturating_add(1);
            drop(state);
            drop(self);
            return Err(ServiceWorkAdmissionError::Closing);
        }
        // Le démarrage Tauri est synchrone : son runtime global est l'unique
        // autorité capable de lancer ce travail avec ou sans runtime Tokio entré.
        let handle = tauri::async_runtime::spawn(async move {
            drop(self.run(work(cancellation)).await);
            let _ = completed.send(());
        });
        let slot = &mut state.slots[key.index];
        if !slot.occupied || slot.generation != key.generation {
            drop(state);
            handle.abort();
            return Err(ServiceWorkAdmissionError::Closing);
        }
        slot.handle = Some(handle);
        Ok(completion)
    }

    #[cfg(test)]
    pub(in crate::services) fn key_for_test(&self) -> ServiceWorkKey {
        self.key.expect("service work key")
    }
}

impl<const CAPACITY: usize> std::fmt::Debug for ServiceWorkAdmission<CAPACITY> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServiceWorkAdmission")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl<const CAPACITY: usize> Drop for ServiceWorkAdmission<CAPACITY> {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            let _ = self.registry.release(key);
        }
    }
}

impl<const CAPACITY: usize> WorkRegistry<CAPACITY> {
    pub fn try_admit(
        &self,
        app: &AppWorkSupervisor,
    ) -> Result<ServiceWorkAdmission<CAPACITY>, ServiceWorkAdmissionError> {
        let app_admission = app.try_admit().map_err(map_app_error)?;
        let app_cancel = app_admission.cancellation_token();
        let service_cancel = self.inner.cancel.child_token();
        let mut state = self.lock_state();
        if state.phase != ServiceWorkPhase::Open {
            state.diagnostics.closing_refusals =
                state.diagnostics.closing_refusals.saturating_add(1);
            return Err(ServiceWorkAdmissionError::Closing);
        }
        let Some(index) = state.slots.iter().position(|slot| !slot.occupied) else {
            state.diagnostics.saturation_refusals =
                state.diagnostics.saturation_refusals.saturating_add(1);
            return Err(ServiceWorkAdmissionError::Capacity);
        };
        let slot = &mut state.slots[index];
        slot.generation = next_generation(slot.generation);
        slot.occupied = true;
        debug_assert!(slot.app_admission.is_none());
        slot.app_admission = Some(app_admission);
        drop(slot.handle.take());
        let key = ServiceWorkKey {
            index,
            generation: slot.generation,
        };
        state.diagnostics.active += 1;
        state.diagnostics.high_water = state.diagnostics.high_water.max(state.diagnostics.active);
        drop(state);
        let cancellation = ServiceWorkCancellation {
            app: app_cancel,
            service: service_cancel,
        };
        Ok(ServiceWorkAdmission {
            registry: self.clone(),
            key: Some(key),
            cancellation,
        })
    }

    pub fn spawn<Factory, Task>(
        &self,
        app: &AppWorkSupervisor,
        work: Factory,
    ) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.try_admit(app)?.spawn(work)
    }
}

impl<const CAPACITY: usize> ServiceWorkSupervisor<CAPACITY> {
    pub fn new(app: AppWorkSupervisor) -> Self {
        Self {
            app,
            registry: WorkRegistry::new(),
        }
    }

    pub fn try_admit(&self) -> Result<ServiceWorkAdmission<CAPACITY>, ServiceWorkAdmissionError> {
        self.registry.try_admit(&self.app)
    }

    pub fn try_probe(&self) -> Result<(), ServiceWorkAdmissionError> {
        let app_admission = self.app.try_admit().map_err(map_app_error)?;
        let mut state = self.registry.lock_state();
        if state.phase != ServiceWorkPhase::Open {
            state.diagnostics.closing_refusals =
                state.diagnostics.closing_refusals.saturating_add(1);
            return Err(ServiceWorkAdmissionError::Closing);
        }
        drop(state);
        drop(app_admission);
        Ok(())
    }

    pub fn spawn<Factory, Task>(&self, work: Factory) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.registry.spawn(&self.app, work)
    }
}

fn map_app_error(error: AppWorkAdmissionError) -> ServiceWorkAdmissionError {
    match error {
        AppWorkAdmissionError::Closing => ServiceWorkAdmissionError::AppClosing,
        AppWorkAdmissionError::Capacity => ServiceWorkAdmissionError::AppCapacity,
    }
}

fn next_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}
