use crate::services::ollama_manager::OllamaManager;
use crate::services::runtime_background::RuntimeBackgroundServices;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

pub fn start(handle: tauri::AppHandle) {
    let background = handle
        .state::<RuntimeBackgroundServices>()
        .inner()
        .clone();
    let manager = handle.state::<OllamaManager>().inner().clone();
    if background
        .spawn_loop(move |cancel| async move {
            let cancellation = CancellationToken::new();
            let mut loop_task = Box::pin(manager.run_background_loop(cancellation.clone()));
            tokio::select! {
                _ = cancel.cancelled() => {
                    cancellation.cancel();
                    loop_task.as_mut().await;
                }
                _ = loop_task.as_mut() => {}
            }
        })
        .is_err()
    {
        ::log::warn!("[ollama] polling unavailable during shutdown");
    }
}
