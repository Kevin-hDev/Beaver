use super::error_codes;
use super::host_process::HostProcess;
use super::runtime::ExtensionRuntime;
use super::types::HostState;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;

pub(crate) const CHANGED_EVENT: &str = "fs:extensions-changed";

pub async fn restart(deadline: Instant) -> Result<bool, String> {
    let runtime = Arc::clone(super::runtime::global()?);
    let work = runtime.work.clone();
    work.run_operation(move |cancel| async move {
        tokio::select! {
            _ = cancel.cancelled() => Err(error_codes::HOST_UNAVAILABLE.to_string()),
            result = runtime.restart_untracked(deadline) => result,
        }
    })
    .await
    .map_err(|error| error.public_code().to_string())?
}

pub async fn retry_load(
    extension_id: String,
    attempts: u8,
    deadline: Instant,
) -> Result<bool, String> {
    let runtime = Arc::clone(super::runtime::global()?);
    let work = runtime.work.clone();
    work.run_operation(move |cancel| async move {
        tokio::select! {
            _ = cancel.cancelled() => Err(error_codes::HOST_UNAVAILABLE.to_string()),
            result = runtime.retry_untracked(extension_id, attempts, deadline) => result,
        }
    })
    .await
    .map_err(|error| error.public_code().to_string())?
}

pub async fn stop_and_wait(deadline: Instant) -> bool {
    let Ok(runtime) = super::runtime::global() else {
        return true;
    };
    stop_runtime(runtime, deadline).await
}

async fn stop_runtime(runtime: &ExtensionRuntime, deadline: Instant) -> bool {
    runtime.work.begin_closing();
    let hosts_stopped = runtime.stop_hosts(deadline).await;
    crate::services::shutdown_completion::combine_with_work(
        hosts_stopped,
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
            result = runtime.start_untracked(new_stop_deadline()) => result,
        };
        let _ = app.emit(CHANGED_EVENT, ());
    })
    .map_err(|error| error.public_code().to_string())
}

pub(super) async fn ensure_running(
    extension_id: &str,
    deadline: Instant,
) -> Result<Arc<HostProcess>, String> {
    let runtime = super::runtime::global()?;
    if !runtime.work.is_open() {
        return Err(error_codes::HOST_UNAVAILABLE.to_string());
    }
    if let Ok((_, _, process)) = runtime.process_for_extension(extension_id, deadline).await {
        return Ok(process);
    }
    let _ = super::registry::find(extension_id)?;
    runtime.set_state(HostState::Starting, None, 0);
    let _ = runtime.sync_hosts_automatically(deadline).await?;
    runtime
        .process_for_extension(extension_id, deadline)
        .await
        .map(|(_, _, process)| process)
}

impl ExtensionRuntime {
    pub(super) async fn start_untracked(&self, deadline: Instant) -> Result<bool, String> {
        if !self.work.is_open() {
            return Err(error_codes::HOST_UNAVAILABLE.to_string());
        }
        self.set_state(HostState::Starting, None, 0);
        let result = self.sync_hosts(deadline).await;
        if result.is_err() {
            self.mark_unavailable().await;
        }
        result
    }

    async fn restart_untracked(&self, deadline: Instant) -> Result<bool, String> {
        self.hosts.lock().await.reset_restart_budgets();
        let stopped = self.stop_hosts_for_restart(deadline).await;
        super::host_stop_boundary::after_confirmed_stop(
            stopped,
            error_codes::HOST_UNAVAILABLE.to_string(),
            async { self.start_untracked(deadline).await },
        )
        .await
    }

    async fn retry_untracked(
        &self,
        extension_id: String,
        attempts: u8,
        deadline: Instant,
    ) -> Result<bool, String> {
        self.hosts.lock().await.reset_restart_budgets();
        self.set_state(HostState::Starting, None, 0);
        let result = self.retry_host_load(extension_id, attempts, deadline).await;
        if result.is_err() {
            self.mark_unavailable().await;
        }
        result
    }

    pub(super) async fn stop_hosts(&self, deadline: Instant) -> bool {
        self.stop_hosts_with_mode(deadline, false).await
    }

    async fn stop_hosts_for_restart(&self, deadline: Instant) -> bool {
        self.stop_hosts_with_mode(deadline, true).await
    }

    async fn stop_hosts_with_mode(&self, deadline: Instant, restarting: bool) -> bool {
        let snapshots = self.hosts.lock().await.begin_stop_all(restarting);
        let mut results = Vec::with_capacity(snapshots.len());
        for (_, _, process) in &snapshots {
            results.push(process.kill(deadline).await);
        }
        let mut hosts = self.hosts.lock().await;
        let mut all_stopped = true;
        for ((identity, generation, _), stopped) in snapshots.into_iter().zip(results) {
            all_stopped &= hosts.remove_stopped(&identity, generation, stopped);
            if !stopped {
                drop(hosts);
                self.mark_stop_unconfirmed(&identity).await;
                hosts = self.hosts.lock().await;
            }
        }
        drop(hosts);
        if !all_stopped {
            return false;
        }
        self.set_state(HostState::Stopped, None, 0);
        true
    }

    async fn mark_unavailable(&self) {
        // Une panne globale invalide tous les canaux. Effacer toutes les autorisations
        // d'extension reste fermé même si le registre est lui-même illisible.
        crate::services::agent_local::permission_gate::clear_all_extensions().await;
        self.set_state(
            HostState::Error,
            Some(error_codes::HOST_UNAVAILABLE.to_string()),
            0,
        );
        super::runtime::mark_enabled_extensions_error();
    }
}

pub(crate) fn new_stop_deadline() -> Instant {
    Instant::now() + Duration::from_millis(super::types::HOST_STOP_TIMEOUT_MS as u64)
}

#[cfg(test)]
#[path = "runtime_lifecycle_tests.rs"]
mod tests;
