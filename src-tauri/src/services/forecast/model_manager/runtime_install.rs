use crate::services::forecast::sidecar_runtime;
use crate::services::model_downloads::{ModelDownloadPhase, ProgressUpdate};
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

pub(super) async fn prepare_runtime(
    sidecar: &Path,
    family_id: &str,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
) -> Result<PathBuf, String> {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(8);
    let directory = sidecar.to_path_buf();
    let family = family_id.to_string();
    let cancellation = cancel.clone();
    let mut task = tokio::task::spawn_blocking(move || {
        sidecar_runtime::prepare_runtime(&directory, &family, &cancellation, |step| {
            let _ = sender.blocking_send(step);
        })
    });
    loop {
        tokio::select! {
            result = &mut task => {
                return result
                    .map_err(|_| "Préparation du runtime Forecast impossible".to_string())?
                    ;
            }
            Some(step) = receiver.recv() => on_progress(runtime_progress(step)),
        }
    }
}

fn runtime_progress(step: sidecar_runtime::RuntimeInstallStep) -> ProgressUpdate {
    let percent = match step {
        sidecar_runtime::RuntimeInstallStep::CreatingEnvironment => 72,
        sidecar_runtime::RuntimeInstallStep::PreparingInstaller => 76,
        sidecar_runtime::RuntimeInstallStep::InstallingDependencies => 80,
        sidecar_runtime::RuntimeInstallStep::Finalizing => 98,
    };
    ProgressUpdate {
        phase: ModelDownloadPhase::PreparingRuntime,
        downloaded: 0,
        total: 0,
        percent,
    }
}
