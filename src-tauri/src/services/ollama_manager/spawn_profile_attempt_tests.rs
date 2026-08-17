use super::spawn_profile::OllamaSpawnAttempt;
use super::spawn_profile_test_support::{env, paths, resolve, FakeResolver, CWD, HOME};
use super::types::OllamaEndpoint;
use std::num::NonZeroU16;
use std::path::Path;

#[test]
fn same_profile_instance_is_reused_without_recalculating_models() {
    let resolver = FakeResolver::with_paths(&paths());
    let models = std::path::PathBuf::from(CWD).join("models");
    let profile = resolve(
        &resolver,
        &[
            ("HOME", HOME),
            ("OLLAMA_MODELS", models.to_str().expect("models")),
        ],
    )
    .expect("profile");
    let before = resolver.calls();
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11434).expect("port"));
    let attempt = OllamaSpawnAttempt::new(&profile, endpoint.clone());
    assert!(std::ptr::eq(attempt.profile(), &profile));
    assert_eq!(attempt.endpoint().as_http_url(), endpoint.as_http_url());
    assert_eq!(attempt.port(), 11434);
    assert_eq!(profile.environment().count("OLLAMA_HOST"), 0);
    assert_eq!(resolver.calls(), before);
}

#[test]
fn endpoint_is_selected_only_when_attempt_is_created_after_work() {
    let resolver = FakeResolver::with_paths(&paths());
    let models = std::path::PathBuf::from(CWD).join("models");
    let profile = super::spawn_profile::OllamaSpawnProfile::resolve(
        &paths(),
        env(&[
            ("HOME", HOME),
            ("OLLAMA_MODELS", models.to_str().expect("models")),
        ]),
        Path::new(CWD),
        &resolver,
    )
    .expect("profile");
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(12345).expect("port"));
    let attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    assert_eq!(attempt.endpoint().as_http_url(), "http://127.0.0.1:12345");
}
