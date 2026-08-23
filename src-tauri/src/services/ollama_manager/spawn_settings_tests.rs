use super::spawn_profile::OllamaSpawnProfile;
use super::spawn_profile_test_support::{paths, FakeResolver, CWD, HOME};
use super::spawn_settings::OllamaSpawnSettings;
use std::ffi::OsString;
use std::path::Path;

fn resolved_environment(
    hardware_accel: &str,
    multi_model: bool,
    inherited: &[(&str, &str)],
) -> super::spawn_environment::FrozenEnvironment {
    let resolver = FakeResolver::with_paths(&paths());
    let settings = OllamaSpawnSettings::from_config(hardware_accel, multi_model);
    OllamaSpawnProfile::resolve_with_overrides(
        &paths(),
        inherited
            .iter()
            .map(|(key, value)| (OsString::from(key), OsString::from(value))),
        Path::new(CWD),
        &resolver,
        settings.environment_overrides(),
    )
    .expect("owned Ollama profile")
    .environment()
    .clone()
}

#[test]
fn single_model_cpu_overrides_inherited_compute_and_residency_settings() {
    let environment = resolved_environment(
        "cpu",
        false,
        &[
            ("HOME", HOME),
            ("OLLAMA_LLM_LIBRARY", "cuda_v13"),
            ("OLLAMA_MAX_LOADED_MODELS", "7"),
        ],
    );

    assert_eq!(environment.get("OLLAMA_LLM_LIBRARY"), Some("cpu"));
    assert_eq!(environment.get("OLLAMA_MAX_LOADED_MODELS"), Some("1"));
    assert_eq!(environment.count("OLLAMA_MAX_LOADED_MODELS"), 1);
}

#[test]
fn single_model_gpu_is_also_limited_to_one_loaded_model() {
    let environment = resolved_environment(
        "gpu",
        false,
        &[("HOME", HOME), ("OLLAMA_MAX_LOADED_MODELS", "7")],
    );

    assert_eq!(environment.get("OLLAMA_LLM_LIBRARY"), Some(""));
    assert_eq!(environment.get("OLLAMA_MAX_LOADED_MODELS"), Some("1"));
}

#[test]
fn multi_model_restores_ollama_automatic_capacity() {
    let environment = resolved_environment(
        "gpu",
        true,
        &[("HOME", HOME), ("OLLAMA_MAX_LOADED_MODELS", "1")],
    );

    assert_eq!(environment.get("OLLAMA_MAX_LOADED_MODELS"), Some("0"));
    assert_eq!(environment.count("OLLAMA_MAX_LOADED_MODELS"), 1);
}
