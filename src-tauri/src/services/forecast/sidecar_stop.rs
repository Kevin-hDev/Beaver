use super::sidecar::{ChronosSidecar, SidecarHandle};
use std::time::{Duration, Instant};

const FORECAST_STOP_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn stop(sidecar: &ChronosSidecar) -> bool {
    stop_state(sidecar, control_deadline()).await
}

pub async fn stop_model(sidecar: &ChronosSidecar, model_id: &str) -> bool {
    let handle = {
        let mut state = sidecar.process.lock().await;
        if state
            .as_ref()
            .is_some_and(|handle| handle.model_id == model_id)
        {
            state.take()
        } else {
            None
        }
    };
    if let Some(handle) = handle {
        let stopped = stop_handle(handle, control_deadline()).await;
        super::sidecar_process::clear_pid_file();
        super::sidecar_http::clear_port();
        sidecar.idle_changed.notify_waiters();
        stopped
    } else {
        true
    }
}

pub(super) async fn stop_state(sidecar: &ChronosSidecar, deadline: Instant) -> bool {
    let Ok(mut process) = tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        sidecar.process.lock(),
    )
    .await
    else {
        return false;
    };
    let handle = process.take();
    drop(process);
    let stopped = match handle {
        Some(handle) => stop_handle(handle, deadline).await,
        None => true,
    };
    super::sidecar_process::clear_pid_file();
    super::sidecar_http::clear_port();
    sidecar.idle_changed.notify_waiters();
    stopped
}

async fn stop_handle(handle: SidecarHandle, deadline: Instant) -> bool {
    matches!(
        tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            super::sidecar_process::kill_child_process(handle.child, handle.pid),
        )
        .await,
        Ok(())
    )
}

fn control_deadline() -> Instant {
    Instant::now() + FORECAST_STOP_TIMEOUT
}

impl ChronosSidecar {
    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        let process_stopped = stop_state(self, deadline).await;
        self.idle_changed.notify_waiters();
        self.work.stop_and_wait(deadline).await && process_stopped
    }

    #[cfg(test)]
    pub(crate) fn try_admit_operation_for_test(&self) -> Result<(), ()> {
        let admission = self.work.try_admit_operation().map_err(|_| ())?;
        drop(admission);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn idle_counts_for_test(&self) -> (usize, usize) {
        self.work.idle_counts_for_test()
    }

    #[cfg(test)]
    pub(crate) async fn start_test_process_for_test(&self) -> Result<u32, String> {
        let admission = self
            .work
            .try_admit_sidecar()
            .map_err(|_| "fixture Forecast indisponible".to_string())?;
        let child = super::sidecar_process::spawn_test_fixture().await?;
        let pid = child.pid();
        *self.process.lock().await = Some(SidecarHandle {
            child: child.publish(),
            pid,
            model_id: "fixture".to_string(),
            family_id: "fixture".to_string(),
            auth_token: zeroize::Zeroizing::new("fixture".to_string()),
            launch: super::sidecar_settings::current(),
            generation: 1,
            publication_generation: self
                .next_publication_generation
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            _admission: admission,
        });
        Ok(pid)
    }

    pub(crate) async fn hold_unpublished_test_process_for_test(
        &self,
        spawned: tokio::sync::oneshot::Sender<u32>,
    ) -> Result<(), String> {
        let _admission = self
            .work
            .try_admit_sidecar()
            .map_err(|_| "fixture Forecast indisponible".to_string())?;
        let child = super::sidecar_process::spawn_test_fixture().await?;
        let _ = spawned.send(child.pid());
        std::future::pending::<()>().await;
        drop(child);
        Ok(())
    }
}
