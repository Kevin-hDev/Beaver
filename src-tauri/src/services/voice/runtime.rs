use super::errors::VoiceError;
use super::types::VoicePhase;
use super::work::{VoiceOwner, VoiceWork, VoiceWorkContext};
use crate::app_exit::AppWorkSupervisor;
#[cfg(any(target_os = "macos", windows))]
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct VoiceRuntime {
    work: VoiceWork,
    #[cfg(any(target_os = "macos", windows))]
    models: super::model::lifecycle::ModelLifecycle,
    #[cfg(any(target_os = "macos", windows))]
    coordinator: Arc<Mutex<super::actions::VoiceCoordinator>>,
}

pub fn new_voice_runtime(app_work: AppWorkSupervisor) -> VoiceRuntime {
    #[cfg(any(target_os = "macos", windows))]
    let coordinator = Arc::new(Mutex::new(super::actions::VoiceCoordinator::default()));
    VoiceRuntime {
        work: VoiceWork::new(app_work.clone()),
        #[cfg(any(target_os = "macos", windows))]
        models: super::model::lifecycle::ModelLifecycle::new_with_coordinator(
            app_work,
            Arc::downgrade(&coordinator),
        ),
        #[cfg(any(target_os = "macos", windows))]
        coordinator,
    }
}

impl VoiceRuntime {
    pub fn try_reserve(&self) -> Result<VoiceReservation, VoiceError> {
        self.work.try_reserve().map(VoiceReservation)
    }

    pub fn spawn_blocking<Work, Output>(
        &self,
        work: Work,
    ) -> Result<oneshot::Receiver<Output>, VoiceError>
    where
        Work: FnOnce(VoiceWorkContext) -> Output + Send + 'static,
        Output: Send + 'static,
    {
        let owner = self.work.try_reserve()?;
        let context = owner.context();
        let (sender, receiver) = oneshot::channel();
        tauri::async_runtime::spawn_blocking(move || {
            let _owner = owner;
            let _ = sender.send(work(context));
        });
        Ok(receiver)
    }

    pub fn phase(&self) -> VoicePhase {
        self.work.phase()
    }

    pub fn begin_closing(&self) {
        self.work.begin_closing();
        #[cfg(any(target_os = "macos", windows))]
        self.models.begin_closing();
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        #[cfg(any(target_os = "macos", windows))]
        {
            let (work, models) = tokio::join!(
                self.work.stop_and_wait(deadline),
                self.models.stop_and_wait(deadline)
            );
            work && models
        }
        #[cfg(target_os = "linux")]
        self.work.stop_and_wait(deadline).await
    }

    #[cfg(any(target_os = "macos", windows))]
    pub(crate) fn models(&self) -> &super::model::lifecycle::ModelLifecycle {
        &self.models
    }

    #[cfg(any(target_os = "macos", windows))]
    pub fn snapshot(&self) -> super::contracts::VoiceSnapshot {
        let mut coordinator = self
            .coordinator
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let now_ms = coordinator.now_ms();
        super::maintenance::sweep(&mut coordinator, now_ms)
    }

    #[cfg(any(target_os = "macos", windows))]
    pub fn dispatch(
        &self,
        action: super::contracts::VoiceAction,
        foreground: bool,
    ) -> Result<super::contracts::VoiceSnapshot, VoiceError> {
        self.dispatch_with_settings(action, foreground, None)
    }

    #[cfg(any(target_os = "macos", windows))]
    pub fn dispatch_with_settings(
        &self,
        action: super::contracts::VoiceAction,
        foreground: bool,
        start_settings: Option<super::types::VoiceSettings>,
    ) -> Result<super::contracts::VoiceSnapshot, VoiceError> {
        use super::contracts::VoiceAction;
        let now_ms = self.now_ms();
        if let VoiceAction::Start {
            destination,
            context_generation,
            language,
        } = action
        {
            let reservation = self.try_reserve()?;
            return self.lock_coordinator().start(
                reservation,
                destination,
                context_generation,
                language,
                start_settings.unwrap_or_default(),
                foreground,
            );
        }
        let mut coordinator = self.lock_coordinator();
        match action {
            VoiceAction::Validate { operation_id } => coordinator.validate(&operation_id),
            VoiceAction::CancelInsertion { operation_id } => {
                coordinator.cancel_insertion(&operation_id)
            }
            VoiceAction::AbandonTrial { trial_id } => coordinator.abandon_trial(&trial_id),
            VoiceAction::DeleteRecovery { recovery_id } => {
                coordinator.delete_recovery(&recovery_id)
            }
            VoiceAction::RestoreRecovery {
                recovery_id,
                draft_key,
            } => coordinator.restore_recovery(&recovery_id, draft_key, now_ms),
            VoiceAction::AcknowledgeDelivery { result_id, outcome } => {
                coordinator.acknowledge(&result_id, outcome, now_ms)
            }
            VoiceAction::DestinationClosed { destination } => {
                coordinator.destination_closed(&destination)
            }
            VoiceAction::MessageAccepted { draft_key, send_id } => {
                coordinator.message_accepted(&draft_key, &send_id)
            }
            VoiceAction::Start { .. }
            | VoiceAction::Install { .. }
            | VoiceAction::Resume { .. }
            | VoiceAction::CancelDownload { .. }
            | VoiceAction::Uninstall { .. } => Err(VoiceError::invalid_transition()),
        }
    }

    #[cfg(any(target_os = "macos", windows))]
    pub(crate) fn remove_model(&self, model_id: &str) -> Result<(), VoiceError> {
        self.models
            .remove(&crate::services::paths::data_dir(), model_id)
            .map_err(|_| VoiceError::configuration_unavailable())
    }

    #[cfg(any(target_os = "macos", windows))]
    fn lock_coordinator(&self) -> std::sync::MutexGuard<'_, super::actions::VoiceCoordinator> {
        self.coordinator
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    #[cfg(any(target_os = "macos", windows))]
    pub(crate) fn coordinator_for_command(
        &self,
    ) -> std::sync::MutexGuard<'_, super::actions::VoiceCoordinator> {
        self.lock_coordinator()
    }

    #[cfg(any(target_os = "macos", windows))]
    fn now_ms(&self) -> u64 {
        self.lock_coordinator().now_ms()
    }

    #[cfg(all(test, any(target_os = "macos", windows)))]
    pub(super) fn coordinator_for_test(
        &self,
    ) -> std::sync::MutexGuard<'_, super::actions::VoiceCoordinator> {
        self.lock_coordinator()
    }

    #[cfg(test)]
    pub(super) fn active_work(&self) -> usize {
        self.work.active()
    }

    #[cfg(test)]
    pub(super) async fn wait_until_inactive_for_test(&self) {
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while self.active_work() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("voice work released");
    }
}

pub struct VoiceReservation(VoiceOwner);

impl VoiceReservation {
    pub fn context(&self) -> VoiceWorkContext {
        self.0.context()
    }
}
