use std::{
    sync::Weak,
    time::{Duration, Instant},
};

use crate::{
    app_exit::AppWorkSupervisor,
    services::{
        voice::types::VoiceUnloadDelay,
        work_registry::{ServiceWorkCancellation, ServiceWorkSupervisor},
    },
};

use super::{
    lifecycle::ModelLifecycle,
    lifecycle_state::{lock, LifecycleInner, LifecycleState, UnloadPlan},
};

impl ModelLifecycle {
    pub fn new(app_work: AppWorkSupervisor) -> Self {
        Self::build(app_work, None)
    }

    pub fn new_with_coordinator(
        app_work: AppWorkSupervisor,
        coordinator: std::sync::Weak<
            std::sync::Mutex<crate::services::voice::actions::VoiceCoordinator>,
        >,
    ) -> Self {
        Self::build(app_work, Some(coordinator))
    }

    fn build(
        app_work: AppWorkSupervisor,
        coordinator: Option<
            std::sync::Weak<std::sync::Mutex<crate::services::voice::actions::VoiceCoordinator>>,
        >,
    ) -> Self {
        let maintenance = ServiceWorkSupervisor::new(app_work);
        let (plan, receiver) = tokio::sync::watch::channel(UnloadPlan {
            generation: 0,
            deadline: None,
        });
        let inner = std::sync::Arc::new(LifecycleInner {
            state: std::sync::Mutex::new(LifecycleState::default()),
            plan,
        });
        if maintenance
            .spawn({
                let inner = std::sync::Arc::downgrade(&inner);
                move |cancel| run(inner, receiver, cancel, coordinator)
            })
            .is_err()
        {
            ::log::warn!("[voice] model maintenance unavailable");
            lock(&inner.state).closing = true;
        }
        Self { inner, maintenance }
    }

    pub fn begin_closing(&self) {
        {
            let mut state = lock(&self.inner.state);
            state.closing = true;
            state.generation = state.generation.wrapping_add(1);
        }
        self.maintenance.begin_closing();
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.maintenance.stop_and_wait(deadline).await
    }
}

pub(super) fn deadline(delay: VoiceUnloadDelay) -> Option<tokio::time::Instant> {
    duration(delay).map(|duration| tokio::time::Instant::now() + duration)
}

pub(super) async fn run(
    inner: Weak<LifecycleInner>,
    mut plans: tokio::sync::watch::Receiver<super::lifecycle_state::UnloadPlan>,
    cancel: ServiceWorkCancellation,
    coordinator: Option<
        std::sync::Weak<std::sync::Mutex<crate::services::voice::actions::VoiceCoordinator>>,
    >,
) {
    loop {
        let plan = *plans.borrow_and_update();
        let changed = async {
            let _ = plans.changed().await;
        };
        let tick = tokio::time::sleep(Duration::from_secs(1));
        tokio::pin!(tick);
        match plan.deadline {
            Some(deadline) => tokio::select! {
                _ = cancel.cancelled() => break,
                _ = changed => continue,
                _ = tokio::time::sleep_until(deadline) => unload(&inner, plan.generation),
                _ = &mut tick => expire(&coordinator),
            },
            None => tokio::select! {
                _ = cancel.cancelled() => break,
                _ = changed => continue,
                _ = &mut tick => expire(&coordinator),
            },
        }
    }
    if let Some(inner) = inner.upgrade() {
        drop(lock(&inner.state).loaded.take());
    }
}

fn expire(
    coordinator: &Option<
        std::sync::Weak<std::sync::Mutex<crate::services::voice::actions::VoiceCoordinator>>,
    >,
) {
    let Some(coordinator) = coordinator.as_ref().and_then(std::sync::Weak::upgrade) else {
        return;
    };
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let now_ms = coordinator.now_ms();
    coordinator.expire_at(now_ms);
}

fn unload(inner: &Weak<LifecycleInner>, generation: u64) {
    let Some(inner) = inner.upgrade() else {
        return;
    };
    let mut state = lock(&inner.state);
    if state.generation == generation && state.occupied.is_none() {
        drop(state.loaded.take());
    }
}

fn duration(delay: VoiceUnloadDelay) -> Option<Duration> {
    match delay {
        VoiceUnloadDelay::Immediately => Some(Duration::ZERO),
        VoiceUnloadDelay::OneMinute => Some(Duration::from_secs(60)),
        VoiceUnloadDelay::TwoMinutes => Some(Duration::from_secs(120)),
        VoiceUnloadDelay::FiveMinutes => Some(Duration::from_secs(300)),
        VoiceUnloadDelay::FifteenMinutes => Some(Duration::from_secs(900)),
        VoiceUnloadDelay::OnExit => None,
    }
}

#[cfg(test)]
pub(super) fn seconds(delay: VoiceUnloadDelay) -> Option<u64> {
    duration(delay).map(|duration| duration.as_secs())
}
