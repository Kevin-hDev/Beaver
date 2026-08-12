use super::sidecar::{ChronosSidecar, SidecarHandle};
use std::time::Instant;

pub async fn stop(sidecar: &ChronosSidecar) {
    stop_state(sidecar).await;
}

pub async fn stop_model(sidecar: &ChronosSidecar, model_id: &str) {
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
        stop_handle(handle).await;
        super::sidecar_process::clear_pid_file();
        super::sidecar_http::clear_port();
        sidecar.idle_changed.notify_waiters();
    }
}

pub(super) async fn stop_state(sidecar: &ChronosSidecar) {
    if let Some(handle) = sidecar.process.lock().await.take() {
        stop_handle(handle).await;
    }
    super::sidecar_process::clear_pid_file();
    super::sidecar_http::clear_port();
    sidecar.idle_changed.notify_waiters();
}

async fn stop_handle(handle: SidecarHandle) {
    let _ = tokio::task::spawn_blocking(move || {
        super::sidecar_process::kill_child_process(handle.child);
    })
    .await;
}

impl ChronosSidecar {
    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        stop_state(self).await;
        self.idle_changed.notify_waiters();
        self.work.stop_and_wait(deadline).await
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
        let child = super::sidecar_process::spawn_test_fixture()?;
        let pid = child.id();
        *self.process.lock().await = Some(SidecarHandle {
            child,
            model_id: "fixture".to_string(),
            family_id: "fixture".to_string(),
            auth_token: zeroize::Zeroizing::new("fixture".to_string()),
            launch: super::sidecar_settings::current(),
            generation: 1,
            _admission: admission,
        });
        Ok(pid)
    }
}
