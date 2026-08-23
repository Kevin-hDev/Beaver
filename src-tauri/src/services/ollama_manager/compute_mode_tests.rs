use super::compute_mode::OllamaComputeMode;
use super::spawn_profile::OllamaSpawnProfile;
use super::spawn_profile_test_support::{paths, FakeResolver, CWD, HOME};
use std::ffi::OsString;
use std::path::Path;

fn resolved_environment(
    setting: &str,
    inherited: &[(&str, &str)],
) -> super::spawn_environment::FrozenEnvironment {
    let resolver = FakeResolver::with_paths(&paths());
    let mode = OllamaComputeMode::from_setting(setting);
    OllamaSpawnProfile::resolve_with_overrides(
        &paths(),
        inherited
            .iter()
            .map(|(key, value)| (OsString::from(key), OsString::from(value))),
        Path::new(CWD),
        &resolver,
        mode.environment_overrides(),
    )
    .expect("owned Ollama profile")
    .environment()
    .clone()
}

#[test]
fn cpu_setting_overrides_an_inherited_gpu_library() {
    let environment =
        resolved_environment("cpu", &[("HOME", HOME), ("OLLAMA_LLM_LIBRARY", "cuda_v13")]);

    assert_eq!(environment.get("OLLAMA_LLM_LIBRARY"), Some("cpu"));
    assert_eq!(environment.count("OLLAMA_LLM_LIBRARY"), 1);
}

#[test]
fn gpu_setting_restores_autodetection_over_an_inherited_cpu_library() {
    let environment = resolved_environment("gpu", &[("HOME", HOME), ("OLLAMA_LLM_LIBRARY", "cpu")]);

    assert_eq!(environment.get("OLLAMA_LLM_LIBRARY"), Some(""));
    assert_eq!(environment.count("OLLAMA_LLM_LIBRARY"), 1);
}

#[test]
fn malformed_setting_falls_back_to_gpu_autodetection() {
    assert_eq!(
        OllamaComputeMode::from_setting("not-a-mode"),
        OllamaComputeMode::Gpu,
    );
}
