use super::error::OllamaErrorCode;
use super::fingerprint::OllamaVersion;
use super::release_source::{
    normalize_remote_version, OllamaArchive, OllamaReleaseManifest, ValidatedHttpsUrl,
};

#[test]
fn remote_version_is_normalized_or_rejected() {
    assert_eq!(
        normalize_remote_version("v1.2.3").unwrap().as_str(),
        "1.2.3"
    );
    assert_eq!(normalize_remote_version("1.2.3").unwrap().as_str(), "1.2.3");
    assert_eq!(
        normalize_remote_version("v1.2").unwrap_err(),
        OllamaErrorCode::OllamaBundleInvalid
    );
}

#[test]
fn manifest_accepts_only_https_allowlisted_release_urls() {
    let version = OllamaVersion::parse("1.2.3").unwrap();
    let url = ValidatedHttpsUrl::parse(
        "https://github.com/ollama/ollama/releases/download/v1.2.3/ollama-darwin.tgz",
    )
    .unwrap();
    let archive = OllamaArchive::new(
        "ollama-darwin.tgz",
        url,
        3,
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();
    assert!(OllamaReleaseManifest::try_new(version, vec![archive]).is_ok());
    assert!(ValidatedHttpsUrl::parse(
        "http://github.com/ollama/ollama/releases/download/v1.2.3/ollama-darwin.tgz"
    )
    .is_err());
    assert!(ValidatedHttpsUrl::parse("https://evil.example/ollama-darwin.tgz").is_err());
}

#[test]
fn manifest_rejects_duplicate_or_oversized_archive_sets() {
    let version = OllamaVersion::parse("1.2.3").unwrap();
    let make = || {
        OllamaArchive::new(
            "ollama-darwin.tgz",
            ValidatedHttpsUrl::parse(
                "https://github.com/ollama/ollama/releases/download/v1.2.3/ollama-darwin.tgz",
            )
            .unwrap(),
            3,
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap()
    };
    assert!(OllamaReleaseManifest::try_new(version.clone(), vec![make(), make()]).is_err());
    assert_eq!(
        OllamaReleaseManifest::try_new(version, vec![]).unwrap_err(),
        OllamaErrorCode::OllamaBundleInvalid
    );
}

#[test]
fn release_source_is_the_only_version_and_fallback_authority() {
    let setup = include_str!("../../commands/ollama_setup.rs");
    let version = include_str!("../../commands/ollama_version.rs");
    assert!(!setup.contains("fallback_ollama_version"));
    assert!(!setup.contains("fetch_latest_github_version"));
    assert!(!version.contains("fetch_latest_github_version"));
    assert!(include_str!("release_source.rs").contains("fetch_latest_version"));
    assert!(include_str!("release_fetch.rs").contains("pub(crate) async fn fetch_latest_version"));
}
