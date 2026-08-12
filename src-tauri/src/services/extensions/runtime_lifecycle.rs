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
    runtime.stop_host().await;
    runtime.work.stop_and_wait(deadline).await
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
        if let Some(process) = slot.take() {
            process.kill().await;
        }
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
            if let Some(host) = process.take() {
                host.kill().await;
            }
            self.mark_unavailable();
        }
        result
    }

    async fn restart_untracked(&self) -> Result<(), String> {
        self.stop_host().await;
        self.auto_restarts.reset();
        self.start_untracked().await
    }

    pub(super) async fn stop_host(&self) {
        if let Some(process) = self.process.lock().await.take() {
            process.kill().await;
        }
        self.set_state(HostState::Stopped, None, 0);
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

#[cfg(test)]
mod tests {
    use super::super::host_paths::HostPaths;
    use super::super::types::ExtensionHostStatus;
    use super::*;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn internal_start_cannot_bypass_closed_admission() {
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("host.mjs");
        std::fs::write(
            &script,
            "import { writeFileSync } from 'node:fs'; writeFileSync('started', 'yes');",
        )
        .unwrap();
        let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
        let work = super::super::work_supervision::ExtensionWorkServices::new(
            coordinator.work_supervisor(),
        );
        work.begin_closing();
        let runtime = ExtensionRuntime {
            paths: Some(HostPaths {
                node: which::which("node").unwrap().canonicalize().unwrap(),
                script,
                directory: directory.path().to_path_buf(),
            }),
            process: Mutex::new(None),
            status: std::sync::RwLock::new(ExtensionHostStatus::default()),
            auto_restarts: super::super::runtime_restart::RestartBudget::default(),
            work,
        };

        assert_eq!(
            runtime.start_untracked().await,
            Err(error_codes::HOST_UNAVAILABLE.to_string())
        );
        assert!(!directory.path().join("started").exists());
        assert!(runtime.process.lock().await.is_none());
    }
}
