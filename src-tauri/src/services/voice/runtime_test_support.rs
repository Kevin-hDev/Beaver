use tokio::sync::oneshot;

use super::{runtime::VoiceRuntime, types::VoicePhase, work::VoiceWorkContext};

impl VoiceRuntime {
    pub fn spawn_blocking<Work, Output>(
        &self,
        work: Work,
    ) -> Result<oneshot::Receiver<Output>, super::errors::VoiceError>
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
}
