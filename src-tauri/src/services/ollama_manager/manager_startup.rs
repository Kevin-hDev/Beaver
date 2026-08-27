use super::recovery::{RecoveryOutcome, RecoveryReason};
use super::recovery_entry::recover_platform_at;
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
        self.run_startup_recovery_at(crate::services::paths::ollama_paths(
            &crate::services::paths::data_dir(),
        ))
        .await
    }

    #[cfg(test)]
    pub(crate) async fn run_startup_recovery_at_for_test(
        &self,
        paths: crate::services::paths::OllamaPaths,
    ) -> StartupBarrierState {
        self.run_startup_recovery_at(paths).await
    }

    pub(super) async fn run_startup_recovery_at(
        &self,
        paths: crate::services::paths::OllamaPaths,
    ) -> StartupBarrierState {
        let guard = match self
            .begin_operation(super::types::OperationState::Recovering)
            .await
        {
            Ok(guard) => guard,
            Err(code) => {
                let state = StartupBarrierState::Blocked { code };
                self.inner().startup.publish(state.clone());
                return state;
            }
        };
        let generation = guard.generation;
        let prepared = tokio::task::spawn_blocking({
            let paths = paths.clone();
            move || super::startup_recovery::prepare(&paths)
        })
        .await
        .map_err(|_| super::error::OllamaErrorCode::OllamaInternal)
        .and_then(|result| result);
        if let Err(code) = prepared {
            let state = StartupBarrierState::Blocked { code };
            self.publish_startup(generation, state.clone());
            guard.fail(code);
            return state;
        }
        loop {
            let outcome = recover_platform_at(paths.clone(), RecoveryReason::Startup).await;
            match outcome {
                Ok(RecoveryOutcome::Ready) => {
                    let completed = tokio::task::spawn_blocking({
                        let paths = paths.clone();
                        move || super::startup_recovery::bundle_state(&paths)
                    })
                    .await
                    .map_err(|_| super::error::OllamaErrorCode::OllamaInternal)
                    .and_then(|result| result);
                    let bundle = match completed {
                        Ok(bundle) => bundle,
                        Err(code) => {
                            let state = StartupBarrierState::Blocked { code };
                            self.publish_startup(generation, state.clone());
                            guard.fail(code);
                            return state;
                        }
                    };
                    self.inner().lock_state().status.bundle = bundle;
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

    pub(super) async fn reconcile_after_operation_error(
        &self,
        paths: crate::services::paths::OllamaPaths,
        code: super::error::OllamaErrorCode,
    ) {
        if matches!(
            self.run_startup_recovery_at(paths).await,
            StartupBarrierState::Ready
        ) {
            self.record_last_error(code);
        }
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
        let mut state = self.inner().lock_state();
        if !matches!(daemon, super::types::DaemonState::Owned { .. }) {
            state.compute_mode = None;
        }
        state.status.daemon = daemon;
    }

    pub(crate) fn publish_owned_daemon(
        &self,
        endpoint: super::types::OllamaEndpoint,
        compute_mode: super::compute_mode::OllamaComputeMode,
    ) {
        let mut state = self.inner().lock_state();
        state.compute_mode = Some(compute_mode);
        state.status.daemon = super::types::DaemonState::Owned { endpoint };
    }

    pub(crate) fn active_compute_mode(&self) -> Option<super::compute_mode::OllamaComputeMode> {
        self.inner().lock_state().compute_mode
    }

    #[cfg(test)]
    pub(crate) fn publish_startup_for_test(&self, generation: u64, state: StartupBarrierState) {
        self.publish_startup(generation, state);
    }

    fn publish_startup(&self, generation: u64, state: StartupBarrierState) {
        if self.inner().lock_state().generation == generation {
            self.inner().startup.publish(state);
        }
    }
}

include!("manager_cancellation.rs");
include!("manager_runtime.rs");
