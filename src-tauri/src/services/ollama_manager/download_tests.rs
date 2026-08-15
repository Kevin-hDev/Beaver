use super::download::{
    bounded_archive_name, download_archives, download_fixture, verify_sha256, DownloadLimits,
};
use super::error::OllamaErrorCode;
use super::fingerprint::Sha256Digest;
use super::release_source::{
    allowlisted_redirect_policy, is_allowlisted_redirect, redirect_pair_is_allowed, OllamaArchive,
    OllamaReleaseManifest,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn archive_name_is_bounded_and_platform_allowlisted() {
    assert!(bounded_archive_name("ollama-darwin.tgz").is_ok());
    assert!(bounded_archive_name("../ollama-darwin.tgz").is_err());
    assert!(bounded_archive_name("/tmp/ollama.tgz").is_err());
    assert!(bounded_archive_name("ollama-unknown.tgz").is_err());
}

#[test]
fn declared_size_must_fit_the_single_archive_budget() {
    let limits = DownloadLimits::default();
    assert!(limits.accepts_declared_size(1));
    assert!(!limits.accepts_declared_size(limits.max_bytes + 1));
    assert_eq!(
        OllamaErrorCode::OllamaDownloadFailed.as_str(),
        "ollama-download-failed"
    );
}

#[test]
fn stream_size_must_match_manifest_exactly() {
    let limits = DownloadLimits::default();
    assert!(limits.accepts_stream_size(32, 32).is_ok());
    assert!(limits.accepts_stream_size(31, 32).is_err());
    assert!(limits.accepts_stream_size(33, 32).is_err());
}

#[test]
fn archive_batch_is_bounded_to_two_temporaries() {
    let limits = DownloadLimits::default();
    assert!(limits.accepts_archive_count(1));
    assert!(limits.accepts_archive_count(2));
    assert!(!limits.accepts_archive_count(0));
    assert!(!limits.accepts_archive_count(3));
}

#[test]
fn redirect_policy_requires_https_and_the_github_allowlist() {
    assert!(is_allowlisted_redirect(
        &Url::parse("https://github.com/ollama/ollama/releases/download/v1.2.3/ollama-darwin.tgz")
            .unwrap()
    ));
    assert!(is_allowlisted_redirect(
        &Url::parse(
            "https://release-assets.githubusercontent.com/github-production-release-asset/123?x=1"
        )
        .unwrap()
    ));
    let source =
        Url::parse("https://github.com/ollama/ollama/releases/download/v1.2.3/ollama-darwin.tgz")
            .unwrap();
    let asset = Url::parse(
        "https://release-assets.githubusercontent.com/github-production-release-asset/123?x=1",
    )
    .unwrap();
    assert!(redirect_pair_is_allowed(&source, &asset));
    assert!(!redirect_pair_is_allowed(
        &source,
        &Url::parse("https://release-assets.githubusercontent.com/asset?x=1").unwrap()
    ));
    assert!(!is_allowlisted_redirect(
        &Url::parse("http://github.com/ollama/ollama/releases/download/v1.2.3/ollama-darwin.tgz")
            .unwrap()
    ));
    assert!(!is_allowlisted_redirect(
        &Url::parse("https://release-assets.githubusercontent.com.evil.test/asset?x=1").unwrap()
    ));
    assert!(!is_allowlisted_redirect(
        &Url::parse("http://release-assets.githubusercontent.com/asset?x=1").unwrap()
    ));
    assert!(!is_allowlisted_redirect(
        &Url::parse("https://release-assets.githubusercontent.com:443/asset?x=1").unwrap()
    ));
    assert!(!is_allowlisted_redirect(
        &Url::parse("https://evil.example/ollama-darwin.tgz").unwrap()
    ));
    let _policy = allowlisted_redirect_policy();
}

#[tokio::test]
async fn downloaded_archive_can_be_extracted_into_empty_staging() {
    let body = b"not-a-real-archive-yet".to_vec();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ollama-darwin.tgz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.clone()))
        .mount(&server)
        .await;
    let root = tempfile::tempdir().unwrap();
    let archive_dir = root.path().join("archive-cache");
    let staging = root.path().join("install-staging");
    std::fs::create_dir(&archive_dir).unwrap();
    std::fs::create_dir(&staging).unwrap();
    let digest = hex::encode(Sha256::digest(&body));
    let version = super::fingerprint::OllamaVersion::parse("1.2.3").unwrap();
    let archive = OllamaArchive::for_test(
        "ollama-darwin.tgz",
        &format!("{}/ollama-darwin.tgz", server.uri()),
        body.len() as u64,
        &digest,
    );
    let manifest = OllamaReleaseManifest::for_test(version, vec![archive]);
    let paths = download_archives(&manifest, &archive_dir, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(paths, vec![archive_dir.join("ollama-darwin.tgz")]);
    assert!(staging.read_dir().unwrap().next().is_none());
    assert_eq!(std::fs::read(&paths[0]).unwrap(), body);
}

#[tokio::test]
async fn local_fixture_download_stream_is_exact_and_durable() {
    let body = b"ollama-fixture-stream".to_vec();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/archive"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.clone()))
        .mount(&server)
        .await;
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("archive.part");
    download_fixture(
        &Url::parse(&format!("{}/archive", server.uri())).unwrap(),
        body.len() as u64,
        &destination,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(std::fs::read(&destination).unwrap(), body);
}

#[tokio::test]
async fn local_fixture_redirect_outside_allowlist_is_rejected_and_removed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302).insert_header("location", "https://evil.example/archive"),
        )
        .mount(&server)
        .await;
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("archive.part");
    let result = download_fixture(
        &Url::parse(&format!("{}/redirect", server.uri())).unwrap(),
        4,
        &destination,
        &CancellationToken::new(),
    )
    .await;
    assert_eq!(result, Err(OllamaErrorCode::OllamaDownloadFailed));
    assert!(!destination.exists());
}

#[test]
fn checksum_is_compared_against_the_complete_download() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("archive.part");
    let bytes = b"complete archive bytes";
    std::fs::write(&path, bytes).unwrap();
    let expected = Sha256Digest::from_hex(&hex::encode(Sha256::digest(bytes))).unwrap();
    assert_eq!(verify_sha256(&path, &expected), Ok(()));
    let wrong =
        Sha256Digest::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap();
    assert_eq!(
        verify_sha256(&path, &wrong),
        Err(OllamaErrorCode::OllamaChecksumMismatch)
    );
}

#[tokio::test]
async fn local_fixture_size_mismatch_never_publishes_a_partial_file() {
    let body = b"too-large".to_vec();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.clone()))
        .mount(&server)
        .await;
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join(PathBuf::from("archive.part"));
    let result = download_fixture(
        &Url::parse(&server.uri()).unwrap(),
        body.len() as u64 - 1,
        &destination,
        &CancellationToken::new(),
    )
    .await;
    assert_eq!(result, Err(OllamaErrorCode::OllamaDownloadFailed));
    assert!(!destination.exists());
}
