use tauri::Manager;

pub(super) struct LiveSpec {
    pub(super) provider: &'static str,
    pub(super) model: &'static str,
    pub(super) region: &'static str,
    pub(super) mode: &'static str,
}

pub(super) const LIVE_SPECS: &[LiveSpec] = &[
    LiveSpec {
        provider: "ollama",
        model: "gemma4:e2b-it-q4_K_M",
        region: "local",
        mode: "auto",
    },
    LiveSpec {
        provider: "ollama",
        model: "qwen3.5:4b",
        region: "local",
        mode: "auto",
    },
    LiveSpec {
        provider: "google",
        model: "gemini-3.5-flash",
        region: "france",
        mode: "medium",
    },
    LiveSpec {
        provider: "mistral",
        model: "mistral-small-2603",
        region: "france",
        mode: "high",
    },
    LiveSpec {
        provider: "openrouter",
        model: "moonshotai/kimi-k2.5",
        region: "france",
        mode: "medium",
    },
    LiveSpec {
        provider: "openai",
        model: "gpt-5.6-luna",
        region: "france",
        mode: "medium",
    },
    LiveSpec {
        provider: "deepseek",
        model: "deepseek-v4-flash",
        region: "france",
        mode: "low",
    },
    LiveSpec {
        provider: "deepseek",
        model: "deepseek-v4-flash",
        region: "france",
        mode: "high",
    },
    LiveSpec {
        provider: "deepseek",
        model: "deepseek-v4-flash",
        region: "france",
        mode: "max",
    },
    LiveSpec {
        provider: "xai",
        model: "grok-4.6",
        region: "france",
        mode: "high",
    },
    LiveSpec {
        provider: "xai-oauth",
        model: "grok-4.6",
        region: "local",
        mode: "high",
    },
    LiveSpec {
        provider: "moonshot",
        model: "kimi-k2.7-code",
        region: "france",
        mode: "auto",
    },
    LiveSpec {
        provider: "zai",
        model: "glm-4.5-flash",
        region: "local",
        mode: "auto",
    },
    LiveSpec {
        provider: "codex-oauth",
        model: "gpt-5.6-luna",
        region: "local",
        mode: "medium",
    },
    LiveSpec {
        provider: "cerebras",
        model: "gpt-oss-120b",
        region: "france",
        mode: "high",
    },
];

pub(super) async fn prepare_ollama(app: &tauri::App) -> Result<(), String> {
    let manager = app
        .state::<crate::services::ollama_manager::OllamaManager>()
        .inner()
        .clone();
    if let Ok(url) = std::env::var("BEAVER_FIXTURE_OLLAMA_URL") {
        let endpoint = crate::services::ollama_manager::OllamaEndpoint::try_from_http_url(&url)
            .map_err(|_| "ollama fixture runtime unavailable".to_string())?;
        manager.publish_external_daemon(endpoint);
        return Ok(());
    }
    if manager.usable_endpoint().await.is_ok() {
        return Ok(());
    }
    if !matches!(
        manager.run_startup_recovery().await,
        crate::services::ollama_manager::StartupBarrierState::Ready
    ) {
        return Err("ollama fixture runtime unavailable".to_string());
    }
    match manager.start().await {
        crate::services::ollama_manager::OllamaStartOutcome::OwnedStarted { .. }
        | crate::services::ollama_manager::OllamaStartOutcome::OwnedAlreadyRunning { .. }
        | crate::services::ollama_manager::OllamaStartOutcome::ExternalAvailable { .. } => Ok(()),
        _ => Err("ollama fixture runtime unavailable".to_string()),
    }
}
