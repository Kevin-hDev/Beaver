use super::sidecar::{ChronosSidecar, SidecarHandle};
use super::sidecar_settings::UnloadPolicy;
use crate::services::work_registry::ServiceWorkCancellation;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn schedule_idle_stop(sidecar: &ChronosSidecar) {
    if touch_state(&sidecar.process).await.is_none() {
        return;
    }
    sidecar.ensure_idle_worker();
    sidecar.idle_changed.notify_waiters();
}

impl ChronosSidecar {
    fn ensure_idle_worker(&self) {
        if self
            .idle_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let state = self.clone();
        if self
            .work
            .spawn_idle(move |cancel| idle_loop(state, cancel))
            .is_err()
        {
            self.idle_started.store(false, Ordering::Release);
        }
    }
}

async fn idle_loop(sidecar: ChronosSidecar, cancel: ServiceWorkCancellation) {
    loop {
        let changed = sidecar.idle_changed.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        match idle_snapshot(&sidecar.process).await {
            Some((generation, UnloadPolicy::After(delay))) => {
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = changed => {}
                    _ = tokio::time::sleep(delay) => {
                        stop_if_generation(&sidecar, generation).await;
                    }
                }
            }
            Some((_, UnloadPolicy::Never)) | None => {
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = changed => {}
                }
            }
        }
    }
}

async fn touch_state(state: &Arc<Mutex<Option<SidecarHandle>>>) -> Option<(u64, UnloadPolicy)> {
    let mut guard = state.lock().await;
    let handle = guard.as_mut()?;
    handle.generation = handle.generation.saturating_add(1);
    Some((handle.generation, handle.launch.unload_policy.clone()))
}

async fn idle_snapshot(state: &Arc<Mutex<Option<SidecarHandle>>>) -> Option<(u64, UnloadPolicy)> {
    let guard = state.lock().await;
    let handle = guard.as_ref()?;
    Some((handle.generation, handle.launch.unload_policy.clone()))
}

async fn stop_if_generation(sidecar: &ChronosSidecar, generation: u64) {
    let should_stop = sidecar
        .process
        .lock()
        .await
        .as_ref()
        .is_some_and(|handle| handle.generation == generation);
    if should_stop {
        super::sidecar_stop::stop_state(sidecar).await;
    }
}
