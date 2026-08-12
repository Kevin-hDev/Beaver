use super::process_identity::ProcessIdentity;
use super::work_registry::{ServiceWorkAdmission, ServiceWorkCancellation, ServiceWorkSupervisor};
use crate::app_exit::AppWorkSupervisor;
use std::future::Future;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Une mise à jour Beaver est exclusive : deux assets concurrents pourraient
// sinon se disputer le même arrêt et le même transfert de helper.
pub const MAX_APP_UPDATE_DOWNLOADS: usize = 1;

type UpdateDownloadWork = ServiceWorkSupervisor<MAX_APP_UPDATE_DOWNLOADS>;

#[derive(Clone)]
pub struct AppUpdateRuntime {
    downloads: UpdateDownloadWork,
    handoff: UpdateHandoff,
}

impl AppUpdateRuntime {
    pub fn new(app: AppWorkSupervisor) -> Self {
        Self {
            downloads: UpdateDownloadWork::new(app),
            handoff: UpdateHandoff::default(),
        }
    }

    pub(crate) fn try_admit(
        &self,
    ) -> Result<ServiceWorkAdmission<MAX_APP_UPDATE_DOWNLOADS>, String> {
        self.downloads.try_admit().map_err(|_| download_error())
    }

    pub async fn run_download<Factory, Work, Output>(&self, work: Factory) -> Result<Output, String>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Work,
        Work: Future<Output = Result<Output, String>>,
    {
        let admission = self.try_admit()?;
        let cancellation = admission.cancellation();
        admission.run(work(cancellation)).await
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.handoff.begin_closing();
        self.downloads.stop_and_wait(deadline).await
    }

    pub(crate) fn handoff(&self) -> &UpdateHandoff {
        &self.handoff
    }

    #[cfg(test)]
    pub(crate) fn diagnostics(&self) -> super::work_registry::ServiceWorkDiagnostics {
        self.downloads.diagnostics()
    }

    pub(crate) fn transferred_identity(&self) -> Option<ProcessIdentity> {
        self.handoff.snapshot()
    }

    #[cfg(test)]
    pub(crate) fn terminate_transferred_for_test(&self) {
        self.handoff.terminate_transferred_for_test();
    }
}

enum HandoffState {
    Open,
    Closing,
    Transferred {
        identity: ProcessIdentity,
        _child: Child,
    },
}

#[derive(Clone)]
pub struct UpdateHandoff(Arc<Mutex<HandoffState>>);

impl Default for UpdateHandoff {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HandoffState::Open)))
    }
}

impl UpdateHandoff {
    pub(crate) fn transfer(
        &self,
        identity: ProcessIdentity,
        child: Child,
        cancellation: &ServiceWorkCancellation,
    ) -> Result<(), Child> {
        let mut state = match self.0.lock() {
            Ok(state) => state,
            Err(_) => return Err(child),
        };
        if cancellation.is_cancelled() || !matches!(*state, HandoffState::Open) {
            return Err(child);
        }
        // Ce remplacement est le point irréversible : validation et admission
        // sont encore vraies sous l'unique verrou du handoff.
        *state = HandoffState::Transferred {
            identity,
            _child: child,
        };
        Ok(())
    }

    fn begin_closing(&self) {
        let Ok(mut state) = self.0.lock() else {
            return;
        };
        if matches!(*state, HandoffState::Open) {
            *state = HandoffState::Closing;
        }
    }

    pub fn snapshot(&self) -> Option<ProcessIdentity> {
        let state = self.0.lock().ok()?;
        match &*state {
            HandoffState::Transferred { identity, .. } => Some(identity.clone()),
            HandoffState::Open | HandoffState::Closing => None,
        }
    }

    #[cfg(test)]
    fn terminate_transferred_for_test(&self) {
        let child = {
            let Ok(mut state) = self.0.lock() else {
                return;
            };
            let previous = std::mem::replace(&mut *state, HandoffState::Closing);
            match previous {
                HandoffState::Transferred { _child: child, .. } => Some(child),
                HandoffState::Open | HandoffState::Closing => None,
            }
        };
        if let Some(mut child) = child {
            super::process_tree::terminate(
                &mut child,
                super::process_tree::ProcessKind::UpdateHelper,
            );
        }
    }
}

fn download_error() -> String {
    "update-download-error".to_string()
}

#[cfg(test)]
#[path = "update_handoff_tests.rs"]
mod tests;
