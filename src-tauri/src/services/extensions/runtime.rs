use super::host_paths::HostPaths;
use super::host_process::HostProcess;
use super::protocol::{HelloResult, SyncResult};
use super::types::{ExtensionHostStatus, HostState, BEAVER_API_VERSION};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock, RwLock};
use tokio::sync::Mutex;

static RUNTIME: OnceLock<Arc<ExtensionRuntime>> = OnceLock::new();

pub struct ExtensionRuntime {
    pub(super) paths: Option<HostPaths>,
    pub(super) process: Mutex<Option<HostProcess>>,
    pub(super) status: RwLock<ExtensionHostStatus>,
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
    start_and_sync().await
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
    async fn sync_locked(&self, slot: &mut Option<HostProcess>) -> Result<(), String> {
        if slot.is_none() {
            let paths = self
                .paths
                .as_ref()
                .ok_or_else(|| "Runtime Node.js indisponible.".to_string())?;
            *slot = Some(HostProcess::spawn(paths)?);
        }
        let process = slot
            .as_mut()
            .ok_or_else(|| "Hôte d'extensions indisponible.".to_string())?;
        let hello = parse::<HelloResult>(process.request("host.hello", json!({})).await?)?;
        if hello.api_version != BEAVER_API_VERSION {
            return Err("Version de l'hôte d'extensions incompatible.".to_string());
        }
        let records = super::registry::enabled_local()?;
        let specifications = super::runtime_sync::build_specs(records)?;
        let response = process
            .request("host.sync", json!({"extensions": specifications}))
            .await
            .and_then(parse::<SyncResult>)?;
        let active = super::runtime_sync::apply(response, &specifications)?;
        self.set_running(hello, active);
        Ok(())
    }

    fn set_running(&self, hello: HelloResult, active: usize) {
        if let Ok(mut status) = self.status.write() {
            status.state = HostState::Running;
            status.node_version = Some(hello.node_version);
            status.jiti_version = hello.jiti_version;
            status.api_version = hello.api_version;
            status.active_extensions = active;
            status.last_error = None;
        }
    }

    pub(super) fn set_state(&self, state: HostState, error: Option<String>, active: usize) {
        if let Ok(mut status) = self.status.write() {
            status.state = state;
            status.active_extensions = active;
            status.last_error = error;
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
    if let Ok(records) = super::registry::enabled_local() {
        for record in records {
            super::registry::mark_error(&record.manifest.id);
        }
    }
}
