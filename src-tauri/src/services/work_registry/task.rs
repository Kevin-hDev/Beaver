use super::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkKey,
    ServiceWorkPhase, WorkRegistry,
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
        let admission = self.try_admit(app)?;
        let key = admission.key.expect("new admission has a key");
        let cancellation = admission.cancellation();
        let mut state = self.lock_state();
        if state.phase != ServiceWorkPhase::Open {
            state.diagnostics.closing_refusals =
                state.diagnostics.closing_refusals.saturating_add(1);
            drop(state);
            drop(admission);
            return Err(ServiceWorkAdmissionError::Closing);
        }
        let handle = tokio::spawn(async move {
            drop(admission.run(work(cancellation)).await);
        });
        let slot = &mut state.slots[key.index];
        if !slot.occupied || slot.generation != key.generation {
            drop(state);
            handle.abort();
            return Err(ServiceWorkAdmissionError::Closing);
        }
        slot.handle = Some(handle);
        Ok(())
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
