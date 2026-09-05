use super::host_identity::HostIdentity;
use super::host_paths::HostPaths;
use super::host_process::HostProcess;
use super::runtime_hosts::RuntimeHosts;
use super::types::{ExtensionHostStatus, HostState};
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
    pub(super) ui_catalog: super::ui_catalog::UiCatalog,
    pub(super) install_jobs: super::install_jobs::InstallJobStore,
    pub(super) work: super::work_supervision::ExtensionWorkServices,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StopHostOutcome {
    Absent,
    Confirmed,
    Unconfirmed,
}

pub fn init(app: &tauri::AppHandle, app_work: AppWorkSupervisor) -> Result<(), String> {
    super::registry::init()?;
    let paths = super::host_paths::resolve(app).ok();
    let mut status = ExtensionHostStatus::default();
    if paths.is_none() {
        status.state = HostState::Error;
        status.last_error = Some(super::error_codes::RUNTIME_UNAVAILABLE.to_string());
    }
    let temporary_root = crate::services::paths::data_dir().join("extension-host-channels");
    let (hosts, exit_receiver) = RuntimeHosts::with_app(temporary_root, app.clone())?;
    let work = super::work_supervision::ExtensionWorkServices::new(app_work);
    let runtime = Arc::new(ExtensionRuntime {
        paths,
        hosts: Mutex::new(hosts),
        sync: Mutex::new(()),
        status: RwLock::new(status),
        ui_catalog: super::ui_catalog::UiCatalog::with_app(app.clone()),
        install_jobs: super::install_jobs::InstallJobStore::production(work.clone(), app.clone()),
        work,
    });
    runtime.start_exit_monitor(exit_receiver)?;
    RUNTIME
        .set(runtime)
        .map_err(|_| super::error_codes::OPERATION_FAILED.to_string())
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
        deadline: std::time::Instant,
    ) -> Result<(HostIdentity, u64, Arc<HostProcess>), String> {
        let record = super::registry::find(extension_id)?;
        let identity = HostIdentity::from_record(&record)?;
        if !record.enabled || !record.trusted {
            let _ = self.stop_host(&identity, deadline).await;
            return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
        }
        let (api_level, generation, process) =
            self.hosts
                .lock()
                .await
                .usable_snapshot(&identity)
                .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())?;
        if api_level != record.manifest.api_level || !process.is_alive() {
            let _ = self
                .stop_host_if_current(&identity, Some(&process), deadline, false)
                .await;
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

    pub(super) async fn stop_host(
        &self,
        identity: &HostIdentity,
        deadline: std::time::Instant,
    ) -> StopHostOutcome {
        self.stop_host_if_current(identity, None, deadline, false)
            .await
    }

    pub(super) async fn stop_host_if_current(
        &self,
        identity: &HostIdentity,
        expected: Option<&Arc<HostProcess>>,
        deadline: std::time::Instant,
        restarting: bool,
    ) -> StopHostOutcome {
        let snapshot = {
            let mut hosts = self.hosts.lock().await;
            let Some((_, generation, process)) = hosts.snapshot(identity) else {
                return StopHostOutcome::Absent;
            };
            if expected.is_some_and(|expected| !Arc::ptr_eq(expected, &process))
                || !hosts.begin_stop(identity, generation, restarting)
            {
                return StopHostOutcome::Unconfirmed;
            }
            (generation, process)
        };
        let catalog_retired = self.ui_catalog.retire(identity, snapshot.0).is_ok();
        if !snapshot.1.kill(deadline).await {
            self.mark_stop_unconfirmed(identity).await;
            return StopHostOutcome::Unconfirmed;
        }
        let mut hosts = self.hosts.lock().await;
        let stopped = hosts.remove_stopped(identity, snapshot.0, true);
        drop(hosts);
        if stopped && catalog_retired {
            return StopHostOutcome::Confirmed;
        }
        self.mark_stop_unconfirmed(identity).await;
        StopHostOutcome::Unconfirmed
    }

    pub(super) async fn revoke_extension(
        &self,
        identity: &HostIdentity,
        deadline: std::time::Instant,
    ) -> bool {
        let stopped = matches!(
            self.stop_host(identity, deadline).await,
            StopHostOutcome::Absent | StopHostOutcome::Confirmed
        );
        if stopped {
            // Une révocation appelée par l'utilisateur termine la série de
            // reprises de cet Hôte. Son futur démarrage repart donc à zéro.
            self.hosts.lock().await.forget_restart_budget(identity);
        }
        stopped
    }

    pub(super) async fn mark_stop_unconfirmed(&self, identity: &HostIdentity) {
        let ids = super::registry_sync::mark_identity_stop_unconfirmed(identity);
        for id in ids {
            crate::services::agent_local::permission_gate::clear_extension(&id).await;
        }
        self.set_state(
            HostState::Error,
            Some(super::error_codes::STOP_UNCONFIRMED.to_string()),
            0,
        );
        self.hosts.lock().await.emit_changed();
    }
}

pub(super) async fn call_context(
    identity: &HostIdentity,
    generation: u64,
) -> Result<super::call_context::ExtensionCallContext, String> {
    global()?.call_context(identity, generation).await
}

pub(super) async fn revoke_extension(
    identity: &HostIdentity,
    deadline: std::time::Instant,
) -> Result<(), String> {
    if global()?.revoke_extension(identity, deadline).await {
        Ok(())
    } else {
        Err(super::error_codes::HOST_UNAVAILABLE.to_string())
    }
}

pub(super) fn parse<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|_| super::error_codes::HOST_INCOMPATIBLE.to_string())
}

pub(super) fn global() -> Result<&'static Arc<ExtensionRuntime>, String> {
    RUNTIME
        .get()
        .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())
}
