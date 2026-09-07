pub(super) use crate::commands::reasoning_fixture::live_specs::{LiveSpec, LIVE_SPECS};
use tauri::Manager;

pub(super) const ROUTES_ENV: &str = "BEAVER_FIXTURE_ROUTES";
pub(super) const MODELS_ENV: &str = "BEAVER_FIXTURE_MODELS";
pub(super) const MODES_ENV: &str = "BEAVER_FIXTURE_MODES";
const MAX_SELECTION_VALUES: usize = 16;
const MAX_SELECTION_VALUE_CHARS: usize = 128;

pub(super) fn select_specs(
    routes: Option<&str>,
    models: Option<&str>,
    modes: Option<&str>,
) -> Result<Vec<&'static LiveSpec>, String> {
    let routes = parse_filter(routes)?;
    let models = parse_filter(models)?;
    let modes = parse_filter(modes)?;
    for (values, label) in [(&routes, "route"), (&models, "model"), (&modes, "mode")] {
        if values.iter().any(|value| {
            !LIVE_SPECS.iter().any(|spec| match label {
                "route" => spec.provider == *value,
                "model" => spec.model == *value,
                _ => spec.mode == *value,
            })
        }) {
            return Err("fixture selection invalid".to_string());
        }
    }
    let selected = LIVE_SPECS
        .iter()
        .filter(|spec| {
            routes.contains(&spec.provider)
                && models.contains(&spec.model)
                && modes.contains(&spec.mode)
        })
        .collect::<Vec<_>>();
    (!selected.is_empty())
        .then_some(selected)
        .ok_or_else(|| "fixture selection invalid".to_string())
}

pub(super) fn select_specs_from_environment() -> Result<Vec<&'static LiveSpec>, String> {
    select_specs(
        std::env::var(ROUTES_ENV).ok().as_deref(),
        std::env::var(MODELS_ENV).ok().as_deref(),
        std::env::var(MODES_ENV).ok().as_deref(),
    )
}

fn parse_filter(value: Option<&str>) -> Result<Vec<&str>, String> {
    let value = value.ok_or_else(|| "fixture selection invalid".to_string())?;
    // Bound the input before allocating a collection from an environment value.
    if value.len() > MAX_SELECTION_VALUES * (MAX_SELECTION_VALUE_CHARS + 1) {
        return Err("fixture selection invalid".into());
    }
    let values = value.split(',').map(str::trim).collect::<Vec<_>>();
    if values.is_empty()
        || values.len() > MAX_SELECTION_VALUES
        || values.iter().any(|value| {
            value.is_empty()
                || value.chars().count() > MAX_SELECTION_VALUE_CHARS
                || !value.chars().all(|character| {
                    character.is_ascii_alphanumeric() || "-_./:".contains(character)
                })
        })
        || values
            .iter()
            .enumerate()
            .any(|(index, value)| values[..index].contains(value))
    {
        return Err("fixture selection invalid".to_string());
    }
    Ok(values)
}

pub(super) fn vision_capable(spec: &LiveSpec) -> bool {
    spec.vision
}

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

#[cfg(test)]
mod tests {
    use super::LIVE_SPECS;

    #[test]
    fn documented_new_direct_and_hosted_couples_have_exact_modes() {
        let expected = [
            (
                "google",
                "gemini-3.8-flash",
                ["low", "medium", "high"].as_slice(),
            ),
            ("zai", "glm-5.3-flash", ["low", "high", "max"].as_slice()),
            (
                "openai",
                "gpt-6-astra",
                ["low", "medium", "high", "xhigh", "max"].as_slice(),
            ),
            (
                "openrouter",
                "google/gemini-3.8-flash",
                ["low", "medium", "high"].as_slice(),
            ),
            (
                "openrouter",
                "z-ai/glm-5.3-flash",
                ["low", "high", "max"].as_slice(),
            ),
            (
                "openrouter",
                "openai/gpt-6-astra",
                ["low", "medium", "high", "xhigh", "max"].as_slice(),
            ),
            (
                "ollama",
                "glm-5.3-flash:cloud",
                ["low", "high", "max"].as_slice(),
            ),
            (
                "codex-oauth",
                "gpt-6-astra",
                ["low", "medium", "high", "xhigh", "max", "ultra"].as_slice(),
            ),
        ];
        for (provider, model, modes) in expected {
            let actual = LIVE_SPECS
                .iter()
                .filter(|spec| spec.provider == provider && spec.model == model)
                .map(|spec| spec.mode)
                .collect::<Vec<_>>();
            assert_eq!(actual, modes, "{provider}:{model}");
        }
    }

    #[test]
    fn selection_requires_exact_bounded_filters_and_never_sweeps_old_routes() {
        let selected =
            super::select_specs(Some("google"), Some("gemini-3.8-flash"), Some("low,high"))
                .unwrap();
        assert_eq!(selected.len(), 2);
        assert!(selected.iter().all(|spec| spec.model == "gemini-3.8-flash"));
        assert!(super::select_specs(None, Some("gemini-3.8-flash"), Some("low")).is_err());
        assert!(
            super::select_specs(Some("unknown"), Some("gemini-3.8-flash"), Some("low")).is_err()
        );
        assert!(
            super::select_specs(Some("google"), Some("gemini-3.8-flash"), Some("max")).is_err()
        );
        assert!(super::select_specs(Some("google"), Some("glm-5.3-flash"), Some("low")).is_err());
        assert!(
            super::select_specs(Some("google"), Some("gemini-3.8-flash"), Some("low,low")).is_err()
        );
    }
}
