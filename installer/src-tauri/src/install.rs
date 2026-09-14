use crate::contract::{InstallerEvent, InstallerSnapshot};
use crate::download::{download_pinned_asset, percent};
use crate::error::InstallerError;
use crate::install_platform as platform;
use crate::install_trace::TraceSession;
use crate::launch_args::LaunchContext;
use crate::runtime::InstallerRuntime;
use crate::temp_ownership::OwnedTempRun;
use crate::trace::Outcome;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;

pub struct InstallerService {
    launch: LaunchContext,
    run: OwnedTempRun,
    destination: Mutex<PathBuf>,
    runtime: Arc<InstallerRuntime>,
    trace: TraceSession,
    cleanup_done: AtomicBool,
}

impl InstallerService {
    pub fn new(launch: LaunchContext, run: OwnedTempRun) -> Result<Self, InstallerError> {
        let destination = platform::default_destination()?;
        let installed = platform::installed_version(&destination);
        let running = platform::beaver_running(&destination)?;
        Ok(Self {
            runtime: Arc::new(InstallerRuntime::new(
                &launch.release.version,
                &destination.to_string_lossy(),
                installed,
                running,
                platform::platform_kind(),
            )),
            launch,
            run,
            destination: Mutex::new(destination),
            trace: TraceSession::new(),
            cleanup_done: AtomicBool::new(false),
        })
    }

    pub fn snapshot(&self) -> Result<InstallerSnapshot, InstallerError> {
        let destination = self.destination()?;
        self.runtime
            .refresh_running(platform::beaver_running(&destination)?)
    }

    pub fn set_destination(
        &self,
        destination: PathBuf,
    ) -> Result<InstallerSnapshot, InstallerError> {
        platform::validate_destination(&destination)?;
        let running = platform::beaver_running(&destination)?;
        let installed = platform::installed_version(&destination);
        let text = destination.to_string_lossy().into_owned();
        let mut selected = self
            .destination
            .lock()
            .map_err(|_| InstallerError::InstallFailed)?;
        let snapshot = self.runtime.update_environment(&text, installed, running)?;
        *selected = destination;
        Ok(snapshot)
    }

    pub fn cancel(&self) -> InstallerEvent {
        self.runtime.cancel();
        self.runtime.snapshot()
    }

    pub fn operation_active(&self) -> bool {
        self.runtime.operation_active()
    }

    pub fn shutdown(&self) -> Result<(), InstallerError> {
        if self.operation_active() {
            return Err(InstallerError::CleanupFailed);
        }
        if self.cleanup_done.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        #[cfg(target_os = "windows")]
        let result = {
            // Windows keeps the locked executable and its ownership marker for next-launch purge.
            self.run.disarm_cleanup();
            std::env::current_exe()
                .map_err(|_| InstallerError::CleanupFailed)
                .and_then(|executable| {
                    crate::platform::windows_cleanup::prepare_self_cleanup(&self.run, &executable)
                })
        };
        #[cfg(not(target_os = "windows"))]
        let result = self.run.cleanup();
        result
    }

    pub fn launch_beaver(&self) -> Result<(), InstallerError> {
        let executable = crate::platform::installed_executable(&self.destination()?)?;
        crate::platform::launch(&executable)
    }

    pub async fn install(&self, channel: Channel<InstallerEvent>) -> Result<(), InstallerError> {
        let destination = self.destination()?;
        self.runtime.update_environment(
            &destination.to_string_lossy(),
            platform::installed_version(&destination),
            platform::beaver_running(&destination)?,
        )?;
        let operation = self.runtime.begin()?;
        let mut checking = self.runtime.snapshot();
        checking.log_key = Some("installer.log.checking".into());
        let _ = channel.send(checking);
        let started = match self.trace.start(self.run.path()) {
            Ok(started) => started,
            Err(error) => {
                let event = self.runtime.fail(operation, error.code())?;
                let cancelled = event.snapshot.phase == crate::contract::InstallerPhase::Cancelled;
                let _ = channel.send(event);
                if cancelled {
                    return Ok(());
                }
                return Err(error);
            }
        };

        let runtime = Arc::clone(&self.runtime);
        let result = async {
            platform::validate_destination(&destination)?;
            let asset =
                download_pinned_asset(&self.launch.release, &self.run, &operation, |progress| {
                    if let Some(value) = percent(progress) {
                        if let Ok(event) = runtime.downloading(value) {
                            let _ = channel.send(event);
                        }
                    }
                })
                .await?;
            let _ = channel.send(runtime.verifying()?);
            let work_dir = self.run.path().to_path_buf();
            let version = self.launch.release.version.clone();
            let blocking_operation = operation.clone();
            let blocking_channel = channel.clone();
            tokio::task::spawn_blocking(move || {
                platform::install_asset(
                    &asset,
                    &work_dir,
                    &destination,
                    &version,
                    &blocking_operation,
                    &runtime,
                    &blocking_channel,
                )
            })
            .await
            .map_err(|_| InstallerError::InstallFailed)?
        }
        .await;

        let event = match result {
            Ok(Some(outcome)) => {
                self.trace.finish(started, Outcome::Succeeded, None);
                self.runtime.succeed(operation, outcome)?
            }
            Ok(None) => {
                self.trace.finish(started, Outcome::Cancelled, None);
                self.runtime.cancelled(operation)?
            }
            Err(error) => {
                let event = self.runtime.fail(operation, error.code())?;
                if event.snapshot.phase == crate::contract::InstallerPhase::Cancelled {
                    self.trace.finish(started, Outcome::Cancelled, None);
                    event
                } else {
                    self.trace.finish(started, Outcome::Failed, Some(error));
                    if let Some(trace) = self.trace.take() {
                        let _ = self.run.preserve_failure_trace(trace);
                    }
                    let _ = channel.send(event);
                    return Err(error);
                }
            }
        };
        let _ = channel.send(event);
        Ok(())
    }

    fn destination(&self) -> Result<PathBuf, InstallerError> {
        self.destination
            .lock()
            .map(|destination| destination.clone())
            .map_err(|_| InstallerError::InstallFailed)
    }
}

impl Drop for InstallerService {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
