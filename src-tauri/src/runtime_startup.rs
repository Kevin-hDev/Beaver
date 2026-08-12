use crate::services::runtime_background::RuntimeBackgroundServices;

pub fn start_recovery(
    background: &RuntimeBackgroundServices,
    startup_cutoff: chrono::DateTime<chrono::Utc>,
) {
    let _ = background.spawn_task(|cancel| async move {
        tokio::select! {
            _ = cancel.cancelled() => {}
            result = crate::services::forecast::notes_cleanup::recover_pending_deletions() => {
                if result.is_err() {
                    ::log::warn!("[forecast] récupération des notes différée");
                }
            }
        }
    });
    // Cette réparation est finie mais peut toucher le disque : elle reste attendue à l'arrêt.
    let _ = background.spawn_task(move |cancel| async move {
        tokio::select! {
            _ = cancel.cancelled() => {}
            _ = crate::services::agent_local::subagent_startup_cleanup::cleanup_orphans(startup_cutoff) => {}
        }
    });
}

pub fn start_ollama(background: &RuntimeBackgroundServices, app: &tauri::AppHandle) {
    if crate::services::ollama_lifecycle::ollama_binary_path().is_err() {
        return;
    }
    let handle = app.clone();
    let _ = background.spawn_task(move |cancel| async move {
        let stop_handle = handle.clone();
        let result = tokio::task::spawn_blocking(move || {
            crate::services::ollama_lifecycle::start_sidecar(&handle)
        })
        .await;
        if cancel.is_cancelled() {
            crate::services::ollama_lifecycle::stop_sidecar(&stop_handle);
            return;
        }
        match result {
            Ok(Err(error)) => ::log::error!("[ollama] sidecar start failed: {error}"),
            Err(error) => ::log::error!("[ollama] sidecar task failed: {error}"),
            _ => {}
        }
    });
}

pub fn start_litellm(background: &RuntimeBackgroundServices) {
    let _ = background.spawn_task(|cancel| async move {
        tokio::select! {
            _ = cancel.cancelled() => {}
            _ = crate::services::llm::litellm_catalog::init() => {}
        }
    });
}
