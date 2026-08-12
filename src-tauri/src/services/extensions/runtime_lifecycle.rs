use super::error_codes;
use super::host_process::HostProcess;
use super::runtime::ExtensionRuntime;
use super::types::HostState;
use std::sync::Arc;
use std::time::Instant;
use tauri::Emitter;

pub async fn restart() -> Result<(), String> {
    let runtime = Arc::clone(super::runtime::global()?);
    let work = runtime.work.clone();
    work.run_operation(move |cancel| async move {
        tokio::select! {
            _ = cancel.cancelled() => Err(error_codes::HOST_UNAVAILABLE.to_string()),
            result = runtime.restart_untracked() => result,
        }
    })
    .await
    .map_err(|error| error.public_code().to_string())?
}

pub async fn stop_and_wait(deadline: Instant) -> bool {
    let Ok(runtime) = super::runtime::global() else {
        return true;
    };
    // L'admission ferme avant la destruction du processus afin qu'une
    // requête concurrente ne puisse pas recréer l'hôte pendant l'arrêt.
    runtime.work.begin_closing();
    let host_stopped = runtime.stop_host(deadline).await;
    crate::services::shutdown_completion::combine_with_work(
        host_stopped,
        runtime.work.stop_and_wait(deadline),
    )
    .await
}

pub(super) fn start_background(app: tauri::AppHandle) -> Result<(), String> {
    let runtime = Arc::clone(super::runtime::global()?);
    let work = runtime.work.clone();
    work.spawn_operation(move |cancel| async move {
        let _ = tokio::select! {
            _ = cancel.cancelled() => Err(error_codes::HOST_UNAVAILABLE.to_string()),
            result = runtime.start_untracked() => result,
        };
        let _ = app.emit("fs:extensions-changed", ());
    })
    .map_err(|error| error.public_code().to_string())
}

pub(super) async fn ensure_running() -> Result<Arc<HostProcess>, String> {
    let runtime = super::runtime::global()?;
    if !runtime.work.is_open() {
        return Err(error_codes::HOST_UNAVAILABLE.to_string());
    }
    let mut slot = runtime.process.lock().await;
    if let Some(process) = slot.as_ref() {
        return Ok(process.clone());
    }
    if !runtime.auto_restarts.allow() {
        return Err(error_codes::HOST_UNAVAILABLE.to_string());
    }
    runtime.set_state(HostState::Starting, None, 0);
    if let Err(error) = runtime.sync_locked(&mut slot).await {
        let _ = stop_host_slot(&mut slot, super::host_process::stop_deadline()).await;
        runtime.mark_unavailable();
        return Err(error);
    }
    slot.as_ref()
        .cloned()
        .ok_or_else(|| error_codes::HOST_UNAVAILABLE.to_string())
}

impl ExtensionRuntime {
    pub(super) async fn start_untracked(&self) -> Result<(), String> {
        if !self.work.is_open() {
            return Err(error_codes::HOST_UNAVAILABLE.to_string());
        }
        self.set_state(HostState::Starting, None, 0);
        let mut process = self.process.lock().await;
        let result = self.sync_locked(&mut process).await;
        if result.is_err() {
            let _ = stop_host_slot(&mut process, super::host_process::stop_deadline()).await;
            self.mark_unavailable();
        }
        result
    }

    async fn restart_untracked(&self) -> Result<(), String> {
        let stopped = self.stop_host(super::host_process::stop_deadline()).await;
        super::host_stop_boundary::after_confirmed_stop(
            stopped,
            error_codes::HOST_UNAVAILABLE.to_string(),
            async {
                self.auto_restarts.reset();
                self.start_untracked().await
            },
        )
        .await
    }

    pub(super) async fn stop_host(&self, deadline: Instant) -> bool {
        let Ok(mut slot) = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            self.process.lock(),
        )
        .await
        else {
            return false;
        };
        if !stop_host_slot(&mut slot, deadline).await {
            return false;
        }
        drop(slot);
        self.set_state(HostState::Stopped, None, 0);
        true
    }

    fn mark_unavailable(&self) {
        self.set_state(
            HostState::Error,
            Some("Hôte d'extensions indisponible.".to_string()),
            0,
        );
        super::runtime::mark_enabled_extensions_error();
    }
}

pub(super) async fn stop_host_slot(slot: &mut Option<Arc<HostProcess>>, deadline: Instant) -> bool {
    let Some(process) = slot.as_ref() else {
        return true;
    };
    if !process.kill(deadline).await {
        return false;
    }
    // Le slot reste occupé jusqu'à la confirmation de mort : un arrêt
    // incomplet ne peut donc jamais autoriser le spawn d'un second hôte.
    slot.take();
    true
}

#[cfg(test)]
#[path = "runtime_lifecycle_tests.rs"]
mod tests;
