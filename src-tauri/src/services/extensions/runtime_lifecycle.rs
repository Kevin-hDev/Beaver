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
            result = runtime.start_untracked() => result,
        };
        let _ = app.emit("fs:extensions-changed", ());
    })
    .map_err(|error| error.public_code().to_string())
}

pub(super) async fn ensure_running(extension_id: &str) -> Result<Arc<HostProcess>, String> {
    let runtime = super::runtime::global()?;
    if !runtime.work.is_open() {
        return Err(error_codes::HOST_UNAVAILABLE.to_string());
    }
    if let Ok((_, _, process)) = runtime.process_for_extension(extension_id).await {
        return Ok(process);
    }
    if !runtime.auto_restarts.allow() {
        return Err(error_codes::HOST_UNAVAILABLE.to_string());
    }
    runtime.set_state(HostState::Starting, None, 0);
    runtime.sync_hosts().await?;
    runtime
        .process_for_extension(extension_id)
        .await
        .map(|(_, _, process)| process)
}

impl ExtensionRuntime {
    pub(super) async fn start_untracked(&self) -> Result<(), String> {
        if !self.work.is_open() {
            return Err(error_codes::HOST_UNAVAILABLE.to_string());
        }
        self.set_state(HostState::Starting, None, 0);
        let result = self.sync_hosts().await;
        if result.is_err() {
            self.mark_unavailable().await;
        }
        result
    }

    async fn restart_untracked(&self) -> Result<(), String> {
        let stopped = self.stop_hosts(super::host_process::stop_deadline()).await;
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

    pub(super) async fn stop_hosts(&self, deadline: Instant) -> bool {
        let snapshots = self.hosts.lock().await.revoke_all();
        let mut results = Vec::with_capacity(snapshots.len());
        for (_, _, process) in &snapshots {
            results.push(process.kill(deadline).await);
        }
        let mut hosts = self.hosts.lock().await;
        let mut all_stopped = true;
        for ((identity, generation, _), stopped) in snapshots.into_iter().zip(results) {
            all_stopped &= hosts.remove_stopped(&identity, generation, stopped);
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
            Some("Hôte d'extensions indisponible.".to_string()),
            0,
        );
        super::runtime::mark_enabled_extensions_error();
    }
}

#[cfg(test)]
#[path = "runtime_lifecycle_tests.rs"]
mod tests;
