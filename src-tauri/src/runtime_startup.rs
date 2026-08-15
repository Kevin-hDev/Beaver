use crate::services::runtime_background::RuntimeBackgroundServices;
use tauri::Manager;

pub fn start_recovery(
    background: &RuntimeBackgroundServices,
    startup_cutoff: chrono::DateTime<chrono::Utc>,
) {
    if background
        .spawn_task(|cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => {}
                result = crate::services::forecast::notes_cleanup::recover_pending_deletions() => {
                    if result.is_err() {
                        ::log::warn!("[forecast] récupération des notes différée");
                    }
                }
            }
        })
        .is_err()
    {
        ::log::warn!("[startup] background task unavailable category=forecast-recovery");
    }
    // Cette réparation est finie mais peut toucher le disque : elle reste attendue à l'arrêt.
    if background
        .spawn_task(move |cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => {}
                _ = crate::services::agent_local::subagent_startup_cleanup::cleanup_orphans(startup_cutoff) => {}
            }
        })
        .is_err()
    {
        ::log::warn!("[startup] background task unavailable category=subagent-cleanup");
    }
}

pub fn start_ollama(background: &RuntimeBackgroundServices, app: &tauri::AppHandle) {
    let manager = app
        .state::<crate::services::ollama_manager::OllamaManager>()
        .inner()
        .clone();
    if background
        .spawn_task(move |cancel| async move {
            let barrier = manager.run_startup_recovery().await;
            if !matches!(
                barrier,
                crate::services::ollama_manager::StartupBarrierState::Ready
            ) {
                ::log::warn!("[ollama] startup blocked until recovery succeeds");
                let ready = tokio::select! {
                    _ = cancel.cancelled() => return,
                    state = manager.wait_startup_ready() => state,
                };
                if !matches!(
                    ready,
                    crate::services::ollama_manager::StartupBarrierState::Ready
                ) {
                    return;
                }
            }
            let _ = crate::services::gpu_vram::refresh_owned(cancel.clone()).await;
            if cancel.is_cancelled() {
                return;
            }
            match manager.start().await {
                crate::services::ollama_manager::OllamaStartOutcome::Failed { code }
                | crate::services::ollama_manager::OllamaStartOutcome::BlockedByRecovery { code } =>
                {
                    ::log::error!("[ollama] manager start blocked code={}", code.as_str());
                }
                crate::services::ollama_manager::OllamaStartOutcome::RejectedDuringShutdown => {}
                crate::services::ollama_manager::OllamaStartOutcome::OwnedStarted { .. }
                | crate::services::ollama_manager::OllamaStartOutcome::OwnedAlreadyRunning {
                    ..
                }
                | crate::services::ollama_manager::OllamaStartOutcome::ExternalAvailable {
                    ..
                } => {}
            }
        })
        .is_err()
    {
        ::log::warn!("[startup] background task unavailable category=ollama-start");
    }
}

pub fn start_litellm(background: &RuntimeBackgroundServices) {
    if background
        .spawn_task(|cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => {}
                _ = crate::services::llm::litellm_catalog::init() => {}
            }
        })
        .is_err()
    {
        ::log::warn!("[startup] background task unavailable category=litellm-init");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn startup_admission_refusals_are_not_discarded() {
        let source = include_str!("runtime_startup.rs");

        assert_eq!(source.matches("let _ = background.spawn_task").count(), 1);
        assert!(source.contains("background task unavailable"));
    }
}
