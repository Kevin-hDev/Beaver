use super::errors::VoiceError;
use super::types::VoicePhase;
use super::work::{VoiceOwner, VoiceWork, VoiceWorkContext};
use crate::app_exit::AppWorkSupervisor;
use std::time::Instant;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct VoiceRuntime {
    work: VoiceWork,
    #[cfg(any(target_os = "macos", windows))]
    models: super::model::lifecycle::ModelLifecycle,
}

pub fn new_voice_runtime(app_work: AppWorkSupervisor) -> VoiceRuntime {
    VoiceRuntime {
        work: VoiceWork::new(app_work.clone()),
        #[cfg(any(target_os = "macos", windows))]
        models: super::model::lifecycle::ModelLifecycle::new(app_work),
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
