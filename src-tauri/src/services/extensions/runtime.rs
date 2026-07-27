use super::host_paths::HostPaths;
use super::host_process::HostProcess;
use super::protocol::{HelloResult, SyncResult};
use super::types::{ExtensionDiagnostic, ExtensionHostStatus, HostState, BEAVER_API_VERSION};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock, RwLock};
use tokio::sync::Mutex;

static RUNTIME: OnceLock<Arc<ExtensionRuntime>> = OnceLock::new();

pub struct ExtensionRuntime {
    pub(super) paths: Option<HostPaths>,
    pub(super) process: Mutex<Option<Arc<HostProcess>>>,
    pub(super) status: RwLock<ExtensionHostStatus>,
    auto_restarts: super::runtime_restart::RestartBudget,
}

pub fn init(app: &tauri::AppHandle) -> Result<(), String> {
    super::registry::init()?;
    let paths = super::host_paths::resolve(app).ok();
    let mut status = ExtensionHostStatus::default();
    if paths.is_none() {
        status.state = HostState::Error;
        status.last_error = Some("Runtime Node.js indisponible.".to_string());
    }
    let runtime = Arc::new(ExtensionRuntime {
        paths,
        process: Mutex::new(None),
        status: RwLock::new(status),
        auto_restarts: super::runtime_restart::RestartBudget::default(),
    });
    RUNTIME
        .set(runtime)
        .map_err(|_| "Hôte d'extensions déjà initialisé.".to_string())
}

pub async fn start_and_sync() -> Result<(), String> {
    let runtime = global()?;
    runtime.set_state(HostState::Starting, None, 0);
    let mut process_guard = runtime.process.lock().await;
    let result = runtime.sync_locked(&mut process_guard).await;
    if result.is_err() {
        if let Some(process) = process_guard.take() {
            process.kill().await;
        }
        runtime.set_state(
            HostState::Error,
            Some("Hôte d'extensions indisponible.".to_string()),
            0,
        );
        mark_enabled_extensions_error();
    }
    result
}

pub async fn restart() -> Result<(), String> {
    stop().await;
    if let Ok(runtime) = global() {
        runtime.auto_restarts.reset();
    }
    start_and_sync().await
}

pub(super) async fn ensure_running() -> Result<Arc<HostProcess>, String> {
    let runtime = global()?;
    let mut slot = runtime.process.lock().await;
    if let Some(process) = slot.as_ref() {
        return Ok(process.clone());
    }
    if !runtime.auto_restarts.allow() {
        return Err("Hôte d'extensions indisponible.".to_string());
    }
    runtime.set_state(HostState::Starting, None, 0);
    if let Err(error) = runtime.sync_locked(&mut slot).await {
        if let Some(process) = slot.take() {
            process.kill().await;
        }
        runtime.set_state(
            HostState::Error,
            Some("Hôte d'extensions indisponible.".to_string()),
            0,
        );
        mark_enabled_extensions_error();
        return Err(error);
    }
    slot.as_ref()
        .cloned()
        .ok_or_else(|| "Hôte d'extensions indisponible.".to_string())
}

pub async fn stop() {
    let Ok(runtime) = global() else {
        return;
    };
    if let Some(process) = runtime.process.lock().await.take() {
        process.kill().await;
    }
    runtime.set_state(HostState::Stopped, None, 0);
}

pub fn status() -> ExtensionHostStatus {
    global()
        .ok()
        .and_then(|runtime| runtime.status.read().ok().map(|status| status.clone()))
        .unwrap_or_default()
}

impl ExtensionRuntime {
    async fn sync_locked(&self, slot: &mut Option<Arc<HostProcess>>) -> Result<(), String> {
        if slot.is_none() {
            let paths = self
                .paths
                .as_ref()
                .ok_or_else(|| "Runtime Node.js indisponible.".to_string())?;
            *slot = Some(Arc::new(HostProcess::spawn(paths)?));
        }
        let process = slot
            .as_ref()
            .ok_or_else(|| "Hôte d'extensions indisponible.".to_string())?;
        let hello = parse::<HelloResult>(process.request("host.hello", json!({})).await?)?;
        if hello.api_version != BEAVER_API_VERSION {
            return Err("Version de l'hôte d'extensions incompatible.".to_string());
        }
        let records = super::registry::enabled_hosted()?;
        super::runtime_version::validate_node(&hello.node_version)?;
        let directory = self
            .paths
            .as_ref()
            .ok_or_else(|| "Hôte d'extensions indisponible.".to_string())?
            .directory
            .as_path();
        let build = super::runtime_sync::build_specs(records, directory)?;
        let response = process
            .request("host.sync", json!({"extensions": build.specs}))
            .await
            .and_then(parse::<SyncResult>)?;
        let applied = super::runtime_sync::apply(response, &build)?;
        self.set_running(hello, applied.active, applied.diagnostics);
        Ok(())
    }

    fn set_running(
        &self,
        hello: HelloResult,
        active: usize,
        diagnostics: Vec<ExtensionDiagnostic>,
    ) {
        if let Ok(mut status) = self.status.write() {
            status.state = HostState::Running;
            status.node_version = Some(hello.node_version);
            status.jiti_version = hello.jiti_version;
            status.api_version = hello.api_version;
            status.active_extensions = active;
            status.last_error = None;
            status.diagnostics = diagnostics;
        }
    }

    pub(super) fn set_state(&self, state: HostState, error: Option<String>, active: usize) {
        if let Ok(mut status) = self.status.write() {
            let running = state == HostState::Running;
            status.state = state;
            status.active_extensions = active;
            status.last_error = error;
            if !running {
                status.diagnostics.clear();
            }
        }
    }
}

pub(super) fn parse<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value)
        .map_err(|_| "Réponse de l'hôte d'extensions invalide.".to_string())
}

pub(super) fn global() -> Result<&'static Arc<ExtensionRuntime>, String> {
    RUNTIME
        .get()
        .ok_or_else(|| "Hôte d'extensions indisponible.".to_string())
}

pub(super) fn mark_enabled_extensions_error() {
    super::registry_sync::mark_all_enabled_error();
}
