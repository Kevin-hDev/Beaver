use super::error::OllamaErrorCode;
use super::fingerprint::OllamaVersion;
use super::release_source::{
    normalize_remote_version, OllamaArchive, OllamaReleaseManifest, ValidatedHttpsUrl,
};
use reqwest::StatusCode;

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
fn latest_release_redirect_yields_only_an_exact_ollama_tag() {
    assert_eq!(
        super::release_fetch::version_from_latest_redirect(
            StatusCode::FOUND,
            Some("https://github.com/ollama/ollama/releases/tag/v0.33.1"),
        )
        .unwrap()
        .as_str(),
        "0.33.1"
    );

    for location in [
        None,
        Some("https://github.com/ollama/ollama/releases/latest"),
        Some("https://github.com/other/ollama/releases/tag/v0.33.1"),
        Some("https://evil.example/ollama/ollama/releases/tag/v0.33.1"),
        Some("https://github.com/ollama/ollama/releases/tag/v0.33.1?asset=1"),
    ] {
        assert_eq!(
            super::release_fetch::version_from_latest_redirect(StatusCode::FOUND, location)
                .unwrap_err(),
            OllamaErrorCode::OllamaDownloadFailed
        );
    }
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

#[test]
fn successful_lookup_replaces_the_cached_release() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("ollama-release-cache.json");

    let latest =
        super::release_cache::resolve_at_path(&path, Ok(OllamaVersion::parse("0.33.1").unwrap()))
            .unwrap();

    assert_eq!(latest.as_str(), "0.33.1");
    assert_eq!(
        super::release_cache::read_from_path(&path)
            .unwrap()
            .unwrap()
            .as_str(),
        "0.33.1"
    );
}

#[test]
fn failed_lookup_reuses_only_a_valid_cached_release() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("ollama-release-cache.json");
    super::release_cache::resolve_at_path(&path, Ok(OllamaVersion::parse("0.33.1").unwrap()))
        .unwrap();

    let cached =
        super::release_cache::resolve_at_path(&path, Err(OllamaErrorCode::OllamaDownloadFailed))
            .unwrap();
    assert_eq!(cached.as_str(), "0.33.1");

    std::fs::write(&path, br#"{"schema_version":1,"version":"invalid"}"#).unwrap();
    assert_eq!(
        super::release_cache::resolve_at_path(&path, Err(OllamaErrorCode::OllamaDownloadFailed))
            .unwrap_err(),
        OllamaErrorCode::OllamaDownloadFailed
    );
}

#[test]
fn missing_cache_preserves_the_remote_failure() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("ollama-release-cache.json");

    assert_eq!(
        super::release_cache::resolve_at_path(&path, Err(OllamaErrorCode::OllamaDownloadFailed))
            .unwrap_err(),
        OllamaErrorCode::OllamaDownloadFailed
    );
}
