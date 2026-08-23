use super::bundle_receipt;
use super::durable_fs::platform_fs;
use super::fingerprint::BundleFingerprint;
use super::path_identity_resolver::NativePathIdentityResolver;
use super::port::{DefaultOllamaPortAllocator, OllamaPortAllocator};
use super::process::{DefaultOllamaProcessLauncher, OllamaProcessLauncher, OwnedOllamaProcess};
use super::process_error::map_process_error;
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
                self.publish_external_daemon(endpoint.clone());
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
            Ok(compute_mode) => {
                self.publish_owned_daemon(endpoint.clone(), compute_mode);
                self.publish_bundle_ready();
                OllamaStartOutcome::OwnedStarted { endpoint }
            }
            Err(code) => OllamaStartOutcome::Failed { code },
        }
    }

    async fn spawn_owned(
        &self,
        endpoint: OllamaEndpoint,
    ) -> Result<super::compute_mode::OllamaComputeMode, OllamaErrorCode> {
        let Some(emergency) = self.inner().emergency.clone() else {
            return Err(OllamaErrorCode::OllamaInternal);
        };
        let paths = ollama_paths(&data_dir());
        let receipt =
            bundle_receipt::read_receipt(&platform_fs(), &bundle_receipt_path(&paths.active))?
                .ok_or(OllamaErrorCode::OllamaBundleMissing)?;
        let bundle = receipt.fingerprint;
        let config = match crate::services::config::read_config() {
            Ok(config) => config,
            Err(_) => {
                ::log::error!("[ollama-manager] runtime settings configuration unavailable");
                return Err(OllamaErrorCode::OllamaInternal);
            }
        };
        let spawn_settings = super::spawn_settings::OllamaSpawnSettings::from_config(
            &config.advanced.hardware_accel,
            config.advanced.multi_model,
        );
        let compute_mode = spawn_settings.compute_mode();
        let expected_version = bundle.version.clone();
        let spawn_endpoint = endpoint.clone();
        let process = tokio::task::spawn_blocking(move || {
            spawn_owned_process(paths, bundle, spawn_endpoint, emergency, spawn_settings)
        })
        .await
        .map_err(|_| OllamaErrorCode::OllamaStartFailed)??;
        let deadline = std::time::Instant::now() + super::constants::OWNED_START_TIMEOUT;
        let cancellation = tokio_util::sync::CancellationToken::new();
        let endpoint_ready = matches!(
            super::probe_ownership::wait_for_owned_endpoint(
                &endpoint,
                process.identity(),
                deadline,
                &cancellation,
            )
            .await,
            super::probe_ownership::EndpointWaitResult::Ready
        );
        let version_ready = endpoint_ready
            && super::probe_http::fetch_version(&endpoint, deadline, &cancellation)
                .await
                .is_ok_and(|version| version == expected_version);
        if !version_ready {
            let _ = tokio::task::spawn_blocking(move || {
                process.terminate_and_reap(
                    std::time::Instant::now()
                        + super::constants::PROCESS_REAP_FALLBACK_TIMEOUT,
                )
            })
            .await;
            return Err(OllamaErrorCode::OllamaStartFailed);
        }
        let mut slot = self
            .inner()
            .owned_process
            .lock()
            .map_err(|_| OllamaErrorCode::OllamaInternal)?;
        if slot.is_some() {
            return Err(OllamaErrorCode::OllamaOperationInProgress);
        }
        // L'état Owned n'est publié qu'après l'écoute et la réponse HTTP : le
        // polling ne peut plus rétrograder un démarrage encore en cours.
        *slot = Some(process);
        Ok(compute_mode)
    }

    async fn run_cli_impl(&self, args: OllamaCliArgs) -> Result<OllamaCliOutput, OllamaErrorCode> {
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
        Ok(OllamaCliOutput {
            success: status.success(),
        })
    }
}

fn spawn_owned_process(
    paths: crate::services::paths::OllamaPaths,
    bundle: BundleFingerprint,
    endpoint: OllamaEndpoint,
    emergency: crate::app_exit::AppEmergencyPublisher,
    spawn_settings: super::spawn_settings::OllamaSpawnSettings,
) -> Result<OwnedOllamaProcess, OllamaErrorCode> {
    let cwd = std::env::current_dir().map_err(|error| {
        super::storage_error::io(
            "owned-process-current-directory",
            &error,
            OllamaErrorCode::OllamaStorageUnavailable,
        )
    })?;
    let profile = OllamaSpawnProfile::resolve_with_overrides(
        &paths,
        std::env::vars_os(),
        &cwd,
        &NativePathIdentityResolver,
        spawn_settings.environment_overrides(),
    )?;
    let attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let launcher = DefaultOllamaProcessLauncher::new(bundle);
    let gated = launcher.create_gated(&attempt).map_err(map_process_error)?;
    let receipt = ProcessReceiptStore::platform(paths);
    gated
        .publish(&receipt, &emergency)
        .map_err(map_process_error)
}
