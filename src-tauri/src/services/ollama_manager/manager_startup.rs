use super::recovery::{RecoveryOutcome, RecoveryReason};
use super::recovery_entry::recover_platform;
use super::startup::StartupBarrierState;

impl OllamaManager {
    pub fn begin_closing(&self) {
        // Publier la fermeture avant le registre ferme la fenêtre admission/publication.
        self.inner().retry.close();
        self.inner().startup.publish(StartupBarrierState::Blocked {
            code: super::error::OllamaErrorCode::OllamaClosing,
        });
        self.inner().mark_closing();
        self.inner().work.begin_closing();
    }

    pub async fn run_startup_recovery(&self) -> StartupBarrierState {
        let guard = match self.begin_operation(super::types::OperationState::Recovering).await {
            Ok(guard) => guard,
            Err(code) => {
                let state = StartupBarrierState::Blocked { code };
                self.inner().startup.publish(state.clone());
                return state;
            }
        };
        let generation = guard.generation;
        loop {
            let outcome = recover_platform(RecoveryReason::Startup).await;
            match outcome {
                Ok(RecoveryOutcome::Ready) => {
                    self.publish_startup(generation, StartupBarrierState::Ready);
                    drop(guard);
                    return self.inner().startup.state();
                }
                Ok(RecoveryOutcome::ProgressMade) => {
                    self.inner().retry.reset_after_progress();
                }
                Ok(RecoveryOutcome::Deferred { code }) | Err(code) => {
                    let state = StartupBarrierState::Blocked { code };
                    self.publish_startup(generation, state.clone());
                    if self
                        .inner()
                        .retry
                        .should_log(code, super::retry::RetryCategory::Recovery)
                    {
                        ::log::warn!("[ollama] recovery deferred code={}", code.as_str());
                    }
                    guard.fail(code);
                    return state;
                }
            }
        }
    }

    pub fn request_recovery_retry(&self) -> Result<(), super::error::OllamaErrorCode> {
        self.inner().retry.request_wake()
    }

    pub(crate) fn startup_state(&self) -> StartupBarrierState {
        self.inner().startup.state()
    }

    pub(crate) async fn wait_startup_decision(&self) -> StartupBarrierState {
        self.inner().startup.wait_ready().await
    }

    pub(crate) async fn wait_startup_ready(&self) -> StartupBarrierState {
        self.inner().startup.wait_until_ready().await
    }

    pub(crate) fn retry_handle(&self) -> OllamaRecoveryRetry {
        self.inner().retry.clone()
    }

    pub(crate) fn is_closing(&self) -> bool {
        self.inner().lock_state().closing
    }

    pub(crate) fn publish_daemon(&self, daemon: super::types::DaemonState) {
        self.inner().lock_state().status.daemon = daemon;
    }

    pub(crate) fn publish_startup_for_test(
        &self,
        generation: u64,
        state: StartupBarrierState,
    ) {
        self.publish_startup(generation, state);
    }

    fn publish_startup(&self, generation: u64, state: StartupBarrierState) {
        if self.inner().lock_state().generation == generation {
            self.inner().startup.publish(state);
        }
    }
}
