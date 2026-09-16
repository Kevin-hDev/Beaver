use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

#[cfg(any(target_os = "macos", windows))]
use super::model_downloads_voice::run_voice_download;
use super::{
    model_downloads::{run_forecast_download, run_ollama_download},
    model_downloads_store::ModelDownloadManager,
    model_downloads_types::{ModelDownloadKind, ModelDownloadState},
    work_registry::ServiceWorkCancellation,
};

pub(super) async fn run_one_download(
    app: AppHandle,
    manager: ModelDownloadManager,
    state: ModelDownloadState,
    cancel: CancellationToken,
    shutdown: &ServiceWorkCancellation,
) {
    let shutdown_cancel = cancel.clone();
    let stop = async {
        shutdown.cancelled().await;
        shutdown_cancel.cancel();
    };
    let work = async {
        match state.kind {
            ModelDownloadKind::Ollama => {
                run_ollama_download(app, manager, state, cancel).await;
            }
            ModelDownloadKind::Forecast => {
                run_forecast_download(app, manager, state, cancel).await;
            }
            ModelDownloadKind::Voice => {
                #[cfg(any(target_os = "macos", windows))]
                run_voice_download(app, manager, state, cancel).await;
                #[cfg(target_os = "linux")]
                {
                    let _ = cancel;
                    super::model_downloads::emit_states(
                        &app,
                        manager
                            .finish(
                                &state.id,
                                super::model_downloads_types::ModelDownloadStatus::Failed,
                                Some("model-download-invalid-model"),
                                None,
                            )
                            .await,
                    );
                }
            }
        }
    };
    tokio::pin!(stop);
    tokio::pin!(work);
    tokio::select! {
        _ = &mut stop => work.await,
        _ = &mut work => {}
    }
}
