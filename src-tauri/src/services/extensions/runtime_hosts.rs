use super::host_identity::HostIdentity;
use super::host_process::HostProcess;
use super::types::{ExtensionApiLevel, MAX_HOST_PROCESSES};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) use super::runtime_host_generation::HostGeneration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HostStartReason {
    InitialOrManual,
    Automatic,
}

pub(super) struct HostExitNotice {
    pub(super) identity: HostIdentity,
    pub(super) generation: u64,
    pub(super) kind: super::runtime_host_generation::HostExitKind,
}

impl HostExitNotice {
    pub(super) fn capture(identity: HostIdentity, generation: &Arc<HostGeneration>) -> Self {
        Self {
            identity,
            generation: generation.number,
            kind: generation.exit_kind(),
        }
    }
}

#[path = "runtime_hosts_lifecycle.rs"]
mod lifecycle;
#[path = "runtime_hosts_registry.rs"]
mod registry;
#[path = "runtime_hosts_restart.rs"]
mod restart;
mod snapshots;

pub(super) struct BoundHostChannel {
    pub(super) identity: HostIdentity,
    pub(super) api_level: ExtensionApiLevel,
    pub(super) generation: Arc<HostGeneration>,
    pub(super) process: Arc<HostProcess>,
    revoked: tokio_util::sync::CancellationToken,
    _temporary_directory: tempfile::TempDir,
}

#[derive(Debug)]
pub(super) struct HostReservation {
    identity: HostIdentity,
    generation: Arc<HostGeneration>,
    revoked: tokio_util::sync::CancellationToken,
    temporary_directory: tempfile::TempDir,
    exit_sender: tokio::sync::mpsc::Sender<HostExitNotice>,
}

impl HostReservation {
    pub(super) fn temporary_directory(&self) -> &Path {
        self.temporary_directory.path()
    }

    #[cfg(test)]
    pub(super) fn revoked(&self) -> tokio_util::sync::CancellationToken {
        self.revoked.clone()
    }

    pub(super) fn spawn_binding(&self) -> super::host_process::HostSpawnBinding {
        super::host_process::HostSpawnBinding::new(
            self.identity.clone(),
            Arc::clone(&self.generation),
            self.revoked.clone(),
            self.exit_sender.clone(),
        )
    }
}

pub(super) struct RuntimeHosts {
    pub(super) official: Option<BoundHostChannel>,
    pub(super) third_party: BTreeMap<String, BoundHostChannel>,
    failed: Vec<BoundHostChannel>,
    temporary_root: PathBuf,
    next_generation: u64,
    restart_budgets: BTreeMap<HostIdentity, super::runtime_restart::RestartBudget>,
    exit_sender: tokio::sync::mpsc::Sender<HostExitNotice>,
    app: Option<tauri::AppHandle>,
}

impl RuntimeHosts {
    #[cfg(test)]
    pub(super) fn new(temporary_root: PathBuf) -> Result<Self, String> {
        Self::build(temporary_root, None).map(|(hosts, _)| hosts)
    }

    #[cfg(test)]
    pub(super) fn new_monitored(
        temporary_root: PathBuf,
    ) -> Result<(Self, tokio::sync::mpsc::Receiver<HostExitNotice>), String> {
        Self::build(temporary_root, None)
    }

    pub(super) fn with_app(
        temporary_root: PathBuf,
        app: tauri::AppHandle,
    ) -> Result<(Self, tokio::sync::mpsc::Receiver<HostExitNotice>), String> {
        Self::build(temporary_root, Some(app))
    }

    fn build(
        temporary_root: PathBuf,
        app: Option<tauri::AppHandle>,
    ) -> Result<(Self, tokio::sync::mpsc::Receiver<HostExitNotice>), String> {
        super::runtime_host_storage::purge_orphaned_directories(&temporary_root)?;
        let (exit_sender, exit_receiver) = tokio::sync::mpsc::channel(MAX_HOST_PROCESSES);
        Ok((
            Self {
                official: None,
                third_party: BTreeMap::new(),
                failed: Vec::new(),
                temporary_root,
                next_generation: 1,
                restart_budgets: BTreeMap::new(),
                exit_sender,
                app,
            },
            exit_receiver,
        ))
    }

    pub(super) fn emit_changed(&self) {
        if let Some(app) = &self.app {
            use tauri::Emitter;
            let _ = app.emit(super::runtime_lifecycle::CHANGED_EVENT, ());
        }
    }
}

impl BoundHostChannel {
    fn call_context(&self) -> super::call_context::ExtensionCallContext {
        super::call_context::ExtensionCallContext::from_bound_channel(
            self.identity.clone(),
            self.api_level.clone(),
            self.generation.number,
            self.revoked.clone(),
        )
    }
}
