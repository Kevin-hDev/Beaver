use super::types::{
    BundleState, CancelOutcome, DaemonState, OllamaCliArgs, OllamaCliOutput, OllamaEndpoint,
    OllamaStartOutcome,
};
use crate::services::agent_local::app_handle_global;
use crate::services::background_command;
use std::num::NonZeroU16;
use std::process::Stdio;
use std::time::{Duration, Instant};

impl OllamaManager {
    pub async fn start(&self) -> OllamaStartOutcome {
        if self.is_closing() {
            return OllamaStartOutcome::RejectedDuringShutdown;
        }
        match self.startup_state() {
            super::startup::StartupBarrierState::Pending => {
                if !matches!(self.run_startup_recovery().await, super::startup::StartupBarrierState::Ready) {
                    return OllamaStartOutcome::BlockedByRecovery {
                        code: OllamaErrorCode::OllamaRecoveryDeferred,
                    };
                }
            }
            super::startup::StartupBarrierState::Blocked { code } => {
                return OllamaStartOutcome::BlockedByRecovery { code };
            }
            super::startup::StartupBarrierState::Ready => {}
        }
        let current = self.status().await.daemon;
        if let DaemonState::Owned { endpoint } = current {
            return OllamaStartOutcome::OwnedAlreadyRunning { endpoint };
        }
        let Some(app) = app_handle_global::get() else {
            return OllamaStartOutcome::Failed {
                code: OllamaErrorCode::OllamaInternal,
            };
        };
        match crate::services::ollama_lifecycle::start_sidecar(app) {
            Ok(true) => match current_endpoint() {
                Ok(endpoint) => {
                    self.publish_daemon(DaemonState::Owned { endpoint: endpoint.clone() });
                    self.publish_bundle_ready();
                    OllamaStartOutcome::OwnedStarted { endpoint }
                }
                Err(_) => OllamaStartOutcome::Failed {
                    code: OllamaErrorCode::OllamaStartFailed,
                },
            },
            Ok(false) => match current_endpoint() {
                Ok(endpoint) => {
                    self.publish_daemon(DaemonState::External { endpoint: endpoint.clone() });
                    self.publish_bundle_ready();
                    OllamaStartOutcome::ExternalAvailable { endpoint }
                }
                Err(_) => OllamaStartOutcome::Failed {
                    code: OllamaErrorCode::OllamaUnavailable,
                },
            },
            Err(_) => OllamaStartOutcome::Failed {
                code: OllamaErrorCode::OllamaStartFailed,
            },
        }
    }

    pub async fn restart(&self) -> OllamaStartOutcome {
        if self.is_closing() {
            return OllamaStartOutcome::RejectedDuringShutdown;
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        if self.stop_and_wait(deadline).await.is_err() {
            return OllamaStartOutcome::Failed {
                code: OllamaErrorCode::OllamaStopFailed,
            };
        }
        self.start().await
    }

    pub async fn cancel_operation(&self) -> CancelOutcome {
        if self.is_closing() {
            return CancelOutcome::RejectedDuringShutdown;
        }
        let mut state = self.inner().lock_state();
        if matches!(state.status.operation, OperationState::Idle) {
            return CancelOutcome::AlreadyIdle;
        }
        state.status.operation = OperationState::Cancelling;
        state.status.last_error = Some(OllamaErrorCode::OllamaOperationCancelled);
        CancelOutcome::Cancelled
    }

    pub async fn usable_endpoint(&self) -> Result<OllamaEndpoint, OllamaErrorCode> {
        match self.status().await.daemon {
            DaemonState::Owned { endpoint } | DaemonState::External { endpoint } => Ok(endpoint),
            DaemonState::Unavailable => Err(OllamaErrorCode::OllamaUnavailable),
        }
    }

    pub async fn owned_endpoint(&self) -> Option<OllamaEndpoint> {
        match self.status().await.daemon {
            DaemonState::Owned { endpoint } => Some(endpoint),
            _ => None,
        }
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> Result<(), OllamaErrorCode> {
        if Instant::now() >= deadline {
            return Err(OllamaErrorCode::OllamaSetupTimeout);
        }
        if let Some(app) = app_handle_global::get() {
            crate::services::ollama_lifecycle::stop_sidecar(app);
        }
        self.publish_daemon(DaemonState::Unavailable);
        Ok(())
    }

    pub async fn run_cli(
        &self,
        args: OllamaCliArgs,
    ) -> Result<OllamaCliOutput, OllamaErrorCode> {
        args.validate()?;
        let endpoint = self.usable_endpoint().await?;
        let binary = crate::services::ollama_lifecycle::ollama_binary_path()
            .map_err(|_| OllamaErrorCode::OllamaBundleMissing)?;
        let mut command = background_command::new_tokio(binary);
        match &args {
            OllamaCliArgs::Version => {
                command.arg("--version");
            }
            OllamaCliArgs::Create { model, modelfile } => {
                if self.owned_endpoint().await.is_none() {
                    return Err(OllamaErrorCode::OllamaUnavailable);
                }
                command
                    .args(["create", model, "--file"])
                    .arg(modelfile)
                    .env("OLLAMA_HOST", endpoint.as_http_url());
            }
        }
        let status = tokio::time::timeout(Duration::from_secs(600), command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .status())
            .await
            .map_err(|_| OllamaErrorCode::OllamaSetupTimeout)?
            .map_err(|_| OllamaErrorCode::OllamaStartFailed)?;
        Ok(OllamaCliOutput { success: status.success() })
    }

    fn publish_bundle_ready(&self) {
        self.inner().lock_state().status.bundle = BundleState::Ready;
    }
}

fn current_endpoint() -> Result<OllamaEndpoint, OllamaErrorCode> {
    let port = NonZeroU16::new(crate::services::ollama_port::get_port())
        .ok_or(OllamaErrorCode::OllamaUnavailable)?;
    Ok(OllamaEndpoint::loopback(port))
}
