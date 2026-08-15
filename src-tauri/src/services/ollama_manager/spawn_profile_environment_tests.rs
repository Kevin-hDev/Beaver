use super::error::OllamaErrorCode;
#[cfg(not(windows))]
use super::spawn_profile::MAX_OLLAMA_ENV_TOTAL_UNIX_BYTES;
#[cfg(windows)]
use super::spawn_profile::MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16;
use super::spawn_profile::{
    FrozenEnvironment, OllamaSpawnProfile, MAX_OLLAMA_ENV_ENTRIES, MAX_OLLAMA_ENV_KEY_UNITS,
    MAX_OLLAMA_ENV_VALUE_UNITS,
};
use super::spawn_profile_test_support::{env, paths, resolve, FakeResolver};
use std::ffi::OsString;
use std::path::Path;

#[test]
fn frozen_environment_preserves_unknown_inherited_values_and_overrides_once() {
    let resolver = FakeResolver::with_paths(&paths());
    let profile = resolve(
        &resolver,
        &[
            ("HOME", "/fake/home"),
            ("PATH", "/bin"),
            ("HTTPS_PROXY", "http://proxy.invalid"),
            ("GPU_SELECTOR_UNKNOWN", "cuda:0"),
            ("OLLAMA_HOST", "http://stale.invalid:11434"),
        ],
    )
    .expect("profile");
    let environment = profile.environment();
    assert_eq!(environment.get("HOME"), Some("/fake/home"));
    assert_eq!(environment.get("PATH"), Some("/bin"));
    assert_eq!(environment.get("HTTPS_PROXY"), Some("http://proxy.invalid"));
    assert_eq!(environment.get("GPU_SELECTOR_UNKNOWN"), Some("cuda:0"));
    assert_eq!(environment.count("OLLAMA_HOST"), 0);
    assert_eq!(environment.count("OLLAMA_NO_CLOUD"), 1);
    assert_eq!(environment.count("OLLAMA_MODELS"), 1);
}

#[test]
fn rejects_duplicate_keys_and_all_environment_limits_before_spawn() {
    let resolver = FakeResolver::with_paths(&paths());
    assert_eq!(
        resolve(&resolver, &[("HOME", "/a"), ("HOME", "/b")]),
        Err(OllamaErrorCode::OllamaInternal)
    );
    let entries = (0..=MAX_OLLAMA_ENV_ENTRIES)
        .map(|index| (OsString::from(format!("K{index}")), OsString::from("v")))
        .collect::<Vec<_>>();
    assert_eq!(
        OllamaSpawnProfile::resolve(&paths(), entries, Path::new("/fake/cwd"), &resolver),
        Err(OllamaErrorCode::OllamaInternal)
    );
    let long_key = OsString::from("K".repeat(MAX_OLLAMA_ENV_KEY_UNITS + 1));
    assert_eq!(
        OllamaSpawnProfile::resolve(
            &paths(),
            [(long_key, OsString::from("v"))],
            Path::new("/fake/cwd"),
            &resolver
        ),
        Err(OllamaErrorCode::OllamaInternal)
    );
    let long_value = OsString::from("v".repeat(MAX_OLLAMA_ENV_VALUE_UNITS + 1));
    assert_eq!(
        OllamaSpawnProfile::resolve(
            &paths(),
            [(OsString::from("K"), long_value)],
            Path::new("/fake/cwd"),
            &resolver
        ),
        Err(OllamaErrorCode::OllamaInternal)
    );
}

#[test]
fn rejects_an_unbounded_environment_iterator_before_collecting_it() {
    let resolver = FakeResolver::with_paths(&paths());
    let entries = (0..=MAX_OLLAMA_ENV_ENTRIES)
        .map(|index| (OsString::from(format!("K{index}")), OsString::from("v")));
    assert_eq!(
        OllamaSpawnProfile::resolve(&paths(), entries, Path::new("/fake/cwd"), &resolver),
        Err(OllamaErrorCode::OllamaInternal)
    );
}

#[test]
fn rejects_unix_and_windows_total_environment_limits() {
    let resolver = FakeResolver::with_paths(&paths());
    #[cfg(not(windows))]
    {
        let oversized_unix = OsString::from("x".repeat(MAX_OLLAMA_ENV_TOTAL_UNIX_BYTES));
        assert_eq!(
            OllamaSpawnProfile::resolve(
                &paths(),
                [(OsString::from("K"), oversized_unix)],
                Path::new("/fake/cwd"),
                &resolver
            ),
            Err(OllamaErrorCode::OllamaInternal)
        );
    }
    #[cfg(windows)]
    {
        let oversized_windows = OsString::from("x".repeat(MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16));
        assert_eq!(
            OllamaSpawnProfile::resolve(
                &paths(),
                [(OsString::from("K"), oversized_windows)],
                Path::new("/fake/cwd"),
                &resolver
            ),
            Err(OllamaErrorCode::OllamaInternal)
        );
    }
}

#[test]
fn frozen_environment_accessor_is_bounded_and_does_not_expose_mutation() {
    let environment = FrozenEnvironment::from_entries(env(&[("HOME", "/fake/home")]));
    assert_eq!(environment.entries().len(), 1);
    assert_eq!(environment.get("HOME"), Some("/fake/home"));
}

#[test]
fn dynamic_gpu_overrides_cross_the_explicit_boundary_once() {
    let resolver = FakeResolver::with_paths(&paths());
    let overrides = [(
        OsString::from("OLLAMA_GPU_OVERHEAD"),
        OsString::from("1073741824"),
    )];
    let profile = OllamaSpawnProfile::resolve_with_overrides(
        &paths(),
        env(&[
            ("OLLAMA_MODELS", "/fake/cwd/models"),
            ("GPU_SELECTOR_UNKNOWN", "cuda:0"),
        ]),
        Path::new("/fake/cwd"),
        &resolver,
        overrides,
    )
    .expect("profile with explicit overrides");
    assert_eq!(profile.environment().count("OLLAMA_GPU_OVERHEAD"), 1);
    assert_eq!(
        profile.environment().get("GPU_SELECTOR_UNKNOWN"),
        Some("cuda:0")
    );
}

#[cfg(windows)]
#[test]
fn windows_environment_block_counts_the_final_nul_at_the_exact_limit() {
    use super::spawn_environment::freeze;
    use super::spawn_profile::MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16;

    fn entries_for_total(total: usize) -> Vec<(OsString, OsString)> {
        let keys = ["K0", "K1", "K2", "K3"];
        let body = total - 1;
        let key_and_separators = keys
            .iter()
            .map(|key| key.encode_utf16().count() + 2)
            .sum::<usize>();
        let last_value = body - key_and_separators - (3 * 8_192);
        keys.iter()
            .enumerate()
            .map(|(index, key)| {
                let units = if index == 3 { last_value } else { 8_192 };
                (OsString::from(*key), OsString::from("x".repeat(units)))
            })
            .collect()
    }

    assert!(freeze(
        entries_for_total(MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16 - 1),
        Vec::new()
    )
    .is_ok());
    assert!(freeze(
        entries_for_total(MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16),
        Vec::new()
    )
    .is_ok());
    assert_eq!(
        freeze(
            entries_for_total(MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16 + 1),
            Vec::new()
        ),
        Err(OllamaErrorCode::OllamaInternal)
    );
}
