use super::host_identity::HostIdentity;
use super::host_paths::HostPaths;
use super::host_process::HostProcess;
use super::runtime_hosts::RuntimeHosts;
use super::types::{ExtensionDiagnostic, ExtensionHostStatus, HostState};
use crate::app_exit::AppWorkSupervisor;
use serde_json::Value;
use std::sync::{Arc, OnceLock, RwLock};
use tokio::sync::Mutex;

static RUNTIME: OnceLock<Arc<ExtensionRuntime>> = OnceLock::new();

pub struct ExtensionRuntime {
    pub(super) paths: Option<HostPaths>,
    pub(super) hosts: Mutex<RuntimeHosts>,
    pub(super) sync: Mutex<()>,
    pub(super) status: RwLock<ExtensionHostStatus>,
    pub(super) auto_restarts: super::runtime_restart::RestartBudget,
    pub(super) work: super::work_supervision::ExtensionWorkServices,
}

pub fn init(app: &tauri::AppHandle, app_work: AppWorkSupervisor) -> Result<(), String> {
    super::registry::init()?;
    let paths = super::host_paths::resolve(app).ok();
    let mut status = ExtensionHostStatus::default();
    if paths.is_none() {
        status.state = HostState::Error;
        status.last_error = Some("Runtime Node.js indisponible.".to_string());
    }
    let temporary_root = crate::services::paths::data_dir().join("extension-host-channels");
    let runtime = Arc::new(ExtensionRuntime {
        paths,
        hosts: Mutex::new(RuntimeHosts::new(temporary_root)?),
        sync: Mutex::new(()),
        status: RwLock::new(status),
        auto_restarts: super::runtime_restart::RestartBudget::default(),
        work: super::work_supervision::ExtensionWorkServices::new(app_work),
    });
    RUNTIME
        .set(runtime)
        .map_err(|_| "Hôte d'extensions déjà initialisé.".to_string())
}

pub fn status() -> ExtensionHostStatus {
    global()
        .ok()
        .and_then(|runtime| runtime.status.read().ok().map(|status| status.clone()))
        .unwrap_or_default()
}

impl ExtensionRuntime {
    pub(super) async fn process_for_extension(
        &self,
        extension_id: &str,
    ) -> Result<(HostIdentity, u64, Arc<HostProcess>), String> {
        let record = super::registry::find(extension_id)?;
        let identity = HostIdentity::from_record(&record)?;
        if !record.enabled || !record.trusted {
            let _ = self.stop_channel(&identity, None).await;
            return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
        }
        let (api_level, generation, process) =
            self.hosts
                .lock()
                .await
                .usable_snapshot(&identity)
                .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())?;
        if api_level != record.manifest.api_level || !process.is_alive() {
            let _ = self.stop_channel(&identity, Some(&process)).await;
            return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
        }
        Ok((identity, generation, process))
    }

    pub(super) async fn call_context(
        &self,
        identity: &HostIdentity,
        generation: u64,
    ) -> Result<super::call_context::ExtensionCallContext, String> {
        self.hosts
            .lock()
            .await
            .call_context(identity, generation)
            .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())
    }

    pub(super) async fn stop_channel(
        &self,
        identity: &HostIdentity,
        expected: Option<&Arc<HostProcess>>,
    ) -> bool {
        let Some((_, generation, process)) = self.hosts.lock().await.snapshot(identity) else {
            return true;
        };
        if expected.is_some_and(|expected| !Arc::ptr_eq(expected, &process)) {
            return false;
        }
        if !self.hosts.lock().await.revoke_current(identity, generation) {
            return false;
        }
        if !process.kill(super::host_process::stop_deadline()).await {
            return false;
        }
        self.hosts.lock().await.remove_current(identity, generation)
    }

    pub(super) async fn revoke_extension(&self, identity: &HostIdentity) -> bool {
        let snapshot = {
            let mut hosts = self.hosts.lock().await;
            let Some((_, generation, process)) = hosts.snapshot(identity) else {
                return true;
            };
            let revoked = match identity {
                HostIdentity::ThirdParty(id) => hosts.revoke(id),
                HostIdentity::Official => hosts.revoke_current(identity, generation),
            };
            revoked.then_some((generation, process))
        };
        let Some((generation, process)) = snapshot else {
            return false;
        };
        if !process.kill(super::host_process::stop_deadline()).await {
            return false;
        }
        self.hosts.lock().await.remove_current(identity, generation)
    }

    pub(super) fn set_running(&self, active: usize, diagnostics: Vec<ExtensionDiagnostic>) {
        if let Ok(mut status) = self.status.write() {
            status.state = HostState::Running;
            status.active_extensions = active;
            status.last_error = None;
            status.diagnostics = diagnostics;
        }
    }

    pub(super) fn set_host_version(&self, hello: &super::protocol::HelloResult) {
        if let Ok(mut status) = self.status.write() {
            status.node_version = Some(hello.node_version.clone());
            status.jiti_version = hello.jiti_version.clone();
            status.api_version = hello.api_version.clone();
        }
    }

    pub(super) fn set_state(&self, state: HostState, error: Option<String>, active: usize) {
        if let Ok(mut status) = self.status.write() {
            status.state = state.clone();
            status.active_extensions = active;
            status.last_error = error;
            if state != HostState::Running {
                status.diagnostics.clear();
            }
        }
    }
}

pub(super) async fn call_context(
    identity: &HostIdentity,
    generation: u64,
) -> Result<super::call_context::ExtensionCallContext, String> {
    global()?.call_context(identity, generation).await
}

pub(super) async fn revoke_extension(identity: &HostIdentity) -> Result<(), String> {
    if global()?.revoke_extension(identity).await {
        Ok(())
    } else {
        Err(super::error_codes::HOST_UNAVAILABLE.to_string())
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
