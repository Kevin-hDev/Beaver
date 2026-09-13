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
use std::sync::Mutex;
use tauri::ipc::Channel;

pub struct InstallerService {
    launch: LaunchContext,
    run: OwnedTempRun,
    destination: Mutex<PathBuf>,
    runtime: InstallerRuntime,
    trace: TraceSession,
    cleanup_done: AtomicBool,
}

impl InstallerService {
    pub fn new(launch: LaunchContext, run: OwnedTempRun) -> Result<Self, InstallerError> {
        let destination = platform::default_destination()?;
        let installed = platform::installed_version(&destination);
        let running = platform::beaver_running(&destination)?;
        Ok(Self {
            runtime: InstallerRuntime::new(
                &launch.release.version,
                &destination.to_string_lossy(),
                installed,
                running,
                platform::platform_kind(),
            ),
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

    pub fn set_destination(&self, destination: PathBuf) -> Result<String, InstallerError> {
        platform::validate_destination(&destination)?;
        let running = platform::beaver_running(&destination)?;
        let installed = platform::installed_version(&destination);
        let text = destination.to_string_lossy().into_owned();
        self.runtime.update_environment(&text, installed, running)?;
        *self
            .destination
            .lock()
            .map_err(|_| InstallerError::InstallFailed)? = destination;
        Ok(text)
    }

    pub fn cancel(&self) -> InstallerSnapshot {
        self.runtime.cancel();
        self.runtime.snapshot().snapshot
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
        let result = std::env::current_exe()
            .map_err(|_| InstallerError::CleanupFailed)
            .and_then(|executable| {
                crate::platform::windows_cleanup::schedule_self_cleanup(&self.run, &executable)
            });
        #[cfg(not(target_os = "windows"))]
        let result = self.run.cleanup();
        if result.is_err() {
            self.cleanup_done.store(false, Ordering::Release);
        }
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
        let _ = channel.send(self.runtime.snapshot());
        let started = match self.trace.start(self.run.path()) {
            Ok(started) => started,
            Err(error) => {
                let event = self.runtime.fail(operation, error.code())?;
                let _ = channel.send(event);
                return Err(error);
            }
        };

        let runtime = &self.runtime;
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
            platform::install_asset(
                &asset,
                self.run.path(),
                &destination,
                &self.launch.release.version,
                &operation,
                runtime,
                &channel,
            )
        }
        .await;

        let event = match result {
            Ok(Some(outcome)) => {
                self.trace.finish(started, Outcome::Succeeded, None);
                runtime.succeed(operation, outcome)?
            }
            Ok(None) => {
                self.trace.finish(started, Outcome::Cancelled, None);
                runtime.cancelled(operation)?
            }
            Err(_error) if operation.is_cancelled() => {
                self.trace.finish(started, Outcome::Cancelled, None);
                runtime.cancelled(operation)?
            }
            Err(error) => {
                self.trace.finish(started, Outcome::Failed, Some(error));
                let event = runtime.fail(operation, error.code())?;
                if let Some(trace) = self.trace.take() {
                    let _ = self.run.preserve_failure_trace(trace);
                }
                let _ = channel.send(event);
                return Err(error);
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
