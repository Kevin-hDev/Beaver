use super::bundle_receipt;
use super::durable_fs::platform_fs;
use super::fingerprint::BundleFingerprint;
use super::path_identity_resolver::NativePathIdentityResolver;
use super::port::{DefaultOllamaPortAllocator, OllamaPortAllocator};
use super::process::{
    DefaultOllamaProcessLauncher, OllamaProcessError, OllamaProcessLauncher,
    OwnedOllamaProcess,
};
use super::process_receipt::ProcessReceiptStore;
use super::spawn_profile::{OllamaSpawnAttempt, OllamaSpawnProfile};
use crate::services::background_command;
use crate::services::paths::{bundle_receipt_path, data_dir, ollama_paths};
use std::process::Stdio;

impl OllamaManager {
    async fn start_impl(&self) -> OllamaStartOutcome {
        if self.is_closing() {
            return OllamaStartOutcome::RejectedDuringShutdown;
        }
        match self.startup_state() {
            super::startup::StartupBarrierState::Pending => {
                if !matches!(
                    self.run_startup_recovery().await,
                    super::startup::StartupBarrierState::Ready
                ) {
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

        let _operation = self.inner().operation_lock.lock().await;
        match self.status().await.daemon {
            DaemonState::Owned { endpoint } => {
                return OllamaStartOutcome::OwnedAlreadyRunning { endpoint };
            }
            DaemonState::External { endpoint } => {
                return OllamaStartOutcome::ExternalAvailable { endpoint };
            }
            DaemonState::Unavailable => {}
        }

        let allocator = DefaultOllamaPortAllocator::new();
        match allocator.detect_external().await {
            Ok(Some(endpoint)) => {
                self.publish_daemon(DaemonState::External { endpoint: endpoint.clone() });
                self.publish_bundle_ready();
                return OllamaStartOutcome::ExternalAvailable { endpoint };
            }
            Ok(None) => {}
            Err(code) => return OllamaStartOutcome::Failed { code },
        }
        let endpoint = match allocator.allocate_loopback(&[]) {
            Ok(endpoint) => endpoint,
            Err(code) => return OllamaStartOutcome::Failed { code },
        };

        match self.spawn_owned(endpoint.clone()).await {
            Ok(()) => {
                self.publish_daemon(DaemonState::Owned { endpoint: endpoint.clone() });
                self.publish_bundle_ready();
                OllamaStartOutcome::OwnedStarted { endpoint }
            }
            Err(code) => OllamaStartOutcome::Failed { code },
        }
    }

    async fn spawn_owned(&self, endpoint: OllamaEndpoint) -> Result<(), OllamaErrorCode> {
        let Some(emergency) = self.inner().emergency.clone() else {
            return Err(OllamaErrorCode::OllamaInternal);
        };
        let paths = ollama_paths(&data_dir());
        let receipt = bundle_receipt::read_receipt(
            &platform_fs(),
            &bundle_receipt_path(&paths.active),
        )?
        .ok_or(OllamaErrorCode::OllamaBundleMissing)?;
        let bundle = receipt.fingerprint;
        let process = tokio::task::spawn_blocking(move || {
            spawn_owned_process(paths, bundle, endpoint, emergency)
        })
        .await
        .map_err(|_| OllamaErrorCode::OllamaStartFailed)??;
        let mut slot = self
            .inner()
            .owned_process
            .lock()
            .map_err(|_| OllamaErrorCode::OllamaInternal)?;
        if slot.is_some() {
            return Err(OllamaErrorCode::OllamaOperationInProgress);
        }
        *slot = Some(process);
        Ok(())
    }

    async fn stop_impl(&self, deadline: Instant) -> Result<(), OllamaErrorCode> {
        if Instant::now() >= deadline {
            return Err(OllamaErrorCode::OllamaSetupTimeout);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        let _operation = tokio::time::timeout(remaining, self.inner().operation_lock.lock())
            .await
            .map_err(|_| OllamaErrorCode::OllamaSetupTimeout)?;
        if matches!(self.status().await.daemon, DaemonState::External { .. }) {
            return Ok(());
        }
        let process = self
            .inner()
            .owned_process
            .lock()
            .map_err(|_| OllamaErrorCode::OllamaInternal)?
            .take();
        let Some(process) = process else {
            return Ok(());
        };
        let result = tokio::task::spawn_blocking(move || stop_owned_process(process, deadline))
            .await
            .map_err(|_| OllamaErrorCode::OllamaStopFailed)?;
        match result {
            Ok(()) => {
                self.publish_daemon(DaemonState::Unavailable);
                Ok(())
            }
            Err((process, code)) => {
                self.inner()
                    .owned_process
                    .lock()
                    .map_err(|_| OllamaErrorCode::OllamaInternal)?
                    .replace(process);
                Err(code)
            }
        }
    }

    async fn run_cli_impl(
        &self,
        args: OllamaCliArgs,
    ) -> Result<OllamaCliOutput, OllamaErrorCode> {
        args.validate()?;
        let endpoint = self.usable_endpoint().await?;
        let paths = ollama_paths(&data_dir());
        let binary = super::spawn_profile_paths::active_executable(&paths.active);
        if !binary.is_file() {
            return Err(OllamaErrorCode::OllamaBundleMissing);
        }
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
        };
        let status = tokio::time::timeout(
            Duration::from_secs(600),
            command
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .status(),
        )
        .await
        .map_err(|_| OllamaErrorCode::OllamaSetupTimeout)?
        .map_err(|_| OllamaErrorCode::OllamaStartFailed)?;
        Ok(OllamaCliOutput { success: status.success() })
    }
}

fn spawn_owned_process(
    paths: crate::services::paths::OllamaPaths,
    bundle: BundleFingerprint,
    endpoint: OllamaEndpoint,
    emergency: crate::app_exit::AppEmergencyPublisher,
) -> Result<OwnedOllamaProcess, OllamaErrorCode> {
    let cwd = std::env::current_dir().map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
    let profile = OllamaSpawnProfile::resolve(
        &paths,
        std::env::vars_os(),
        &cwd,
        &NativePathIdentityResolver,
    )?;
    let attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let launcher = DefaultOllamaProcessLauncher::new(bundle);
    let gated = launcher
        .create_gated(&attempt)
        .map_err(map_process_error)?;
    let receipt = ProcessReceiptStore::platform(paths);
    gated.publish(&receipt, &emergency).map_err(map_process_error)
}

fn stop_owned_process(
    mut process: OwnedOllamaProcess,
    deadline: Instant,
) -> Result<(), (OwnedOllamaProcess, OllamaErrorCode)> {
    if let Err(error) = process.terminate() {
        return Err((process, map_process_error(error)));
    }
    if let Err(error) = process.reap(deadline) {
        return Err((process, map_process_error(error)));
    }
    Ok(())
}

fn map_process_error(error: OllamaProcessError) -> OllamaErrorCode {
    match error {
        OllamaProcessError::Receipt => OllamaErrorCode::OllamaStorageUnavailable,
        OllamaProcessError::EmergencyCapacity => OllamaErrorCode::OllamaOperationInProgress,
        OllamaProcessError::Spawn
        | OllamaProcessError::Gate
        | OllamaProcessError::Admission
        | OllamaProcessError::Identity
        | OllamaProcessError::Reap
        | OllamaProcessError::InvalidState => OllamaErrorCode::OllamaStartFailed,
    }
}
