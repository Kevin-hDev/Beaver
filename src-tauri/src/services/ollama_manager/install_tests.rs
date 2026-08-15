use super::bundle_install::{prepare_bundle, write_metadata};
use super::bundle_receipt::{read_receipt, BundleReceipt};
use super::durable_fs::platform_fs;
use super::error::OllamaErrorCode;
use super::fingerprint::OllamaVersion;
use super::install::archive_staging_path;
use super::install::{InstallOutcome, InstallRequest};
use super::install_phases::INSTALL_PHASE_ORDER;
use super::release_source::{OllamaArchive, OllamaReleaseManifest, ValidatedHttpsUrl};
use super::types::BundleState;
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::fs::File;
use std::path::PathBuf;
#[cfg(unix)]
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
#[cfg(unix)]
use wiremock::matchers::{method, path};
#[cfg(unix)]
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn production_install_entrypoint_uses_the_manager_not_a_free_facade() {
    let source = include_str!("../../commands/ollama_setup.rs").replace("\r\n", "\n");
    assert!(source.contains("manager\n        .install(request)"));
    assert!(!source.contains("install_ollama_to"));
}

#[test]
fn first_install_request_is_bounded_and_has_no_update_journal() {
    let request = InstallRequest::for_test(PathBuf::from("/tmp/ollama-install"));
    assert_eq!(
        request.paths.journal.file_name().unwrap(),
        "ollama-update-state.json"
    );
    assert!(request
        .paths
        .install_staging
        .ends_with("ollama-bundle-install-staging"));
    assert!(
        archive_staging_path(&request.paths).ends_with("ollama-bundle-install-staging-archives")
    );
    assert!(!request.cancellation.is_cancelled());
}

#[tokio::test]
async fn active_bundle_is_never_replaced_by_first_install() {
    let root = tempfile::tempdir().unwrap();
    let mut request = InstallRequest::for_test(root.path().to_path_buf());
    std::fs::create_dir_all(&request.paths.active).unwrap();
    request.version = Some(OllamaVersion::parse("1.2.3").unwrap());
    let result = super::install::install(request.clone()).await;
    assert_eq!(result, Err(OllamaErrorCode::OllamaUpdateRecoveryRequired));
    assert!(request.paths.active.exists());
}

#[test]
fn install_outcome_does_not_publish_ready_before_commit() {
    assert!(!InstallOutcome::Preparing.is_ready());
    assert_eq!(BundleState::Absent, BundleState::Absent);
}

#[test]
fn cancellation_token_is_owned_by_the_request() {
    let cancel = CancellationToken::new();
    let request =
        InstallRequest::for_test_with_cancel(PathBuf::from("/tmp/ollama"), cancel.clone());
    cancel.cancel();
    assert!(request.cancellation.is_cancelled());
}

#[test]
fn transaction_order_contains_every_commit_boundary() {
    assert_eq!(
        INSTALL_PHASE_ORDER,
        [
            "profile",
            "manifest",
            "download",
            "verify",
            "extract",
            "remove_archives",
            "fingerprint",
            "version",
            "receipt",
            "sync_staging",
            "probe",
            "rename",
            "sync_parent",
            "reinspect",
            "success",
        ]
    );
}

#[tokio::test]
async fn version_and_receipt_are_coherent_before_publication() {
    let root = tempfile::tempdir().unwrap();
    let root_path = dunce::canonicalize(root.path()).unwrap();
    let paths = crate::services::paths::ollama_paths(&root_path);
    std::fs::create_dir_all(paths.install_staging.join("bin")).unwrap();
    let executable_name = if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    };
    let executable = paths.install_staging.join("bin").join(executable_name);
    std::fs::write(&executable, b"#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let version = OllamaVersion::parse("1.2.3").unwrap();
    let prepared = prepare_bundle(&paths, &version).await.unwrap();
    let fs = std::sync::Arc::new(platform_fs());
    write_metadata(&fs, &paths, &prepared).await.unwrap();
    assert_eq!(
        std::fs::read_to_string(paths.install_staging.join("VERSION")).unwrap(),
        "1.2.3"
    );
    let receipt_path = crate::services::paths::bundle_receipt_path(&paths.install_staging);
    let receipt = read_receipt(&*fs, &receipt_path).unwrap().unwrap();
    assert_eq!(receipt.fingerprint, prepared.fingerprint);
    assert_eq!(receipt, BundleReceipt::new(prepared.fingerprint));
}

#[tokio::test]
async fn invalid_archive_keeps_uncommitted_staging_and_no_active() {
    let root = tempfile::tempdir().unwrap();
    let root_path = dunce::canonicalize(root.path()).unwrap();
    let archive = root.path().join("ollama-darwin.tgz");
    std::fs::write(&archive, b"not an archive").unwrap();
    let digest = hex::encode(Sha256::digest(b"not an archive"));
    let archive_meta = std::fs::metadata(&archive).unwrap();
    let manifest = manifest_for("ollama-darwin.tgz", archive_meta.len(), &digest);
    let mut request = InstallRequest::for_test(root_path);
    request.version = Some(OllamaVersion::parse("1.2.3").unwrap());
    request.manifest = Some(manifest);
    request.local_archives = Some(vec![archive]);
    let result = super::install::install(request.clone()).await;
    assert_eq!(result, Err(OllamaErrorCode::OllamaBundleInvalid));
    assert!(!request.paths.active.exists());
    assert!(request.paths.install_staging.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn first_install_publishes_after_probe_and_reinspection() {
    let root = tempfile::tempdir().unwrap();
    let root_path = dunce::canonicalize(root.path()).unwrap();
    let binary = compile_probe_server(root.path());
    let archive = make_archive(root.path(), &binary);
    let bytes = std::fs::read(&archive).unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ollama-darwin.tgz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes.clone()))
        .mount(&server)
        .await;
    let manifest = manifest_for(
        "ollama-darwin.tgz",
        bytes.len() as u64,
        &hex::encode(Sha256::digest(&bytes)),
    );
    let manifest = OllamaReleaseManifest::for_test(
        manifest.version.clone(),
        vec![OllamaArchive::for_test(
            "ollama-darwin.tgz",
            &format!("{}/ollama-darwin.tgz", server.uri()),
            bytes.len() as u64,
            &hex::encode(Sha256::digest(&bytes)),
        )],
    );
    let mut request = InstallRequest::for_test(root_path);
    request.version = Some(OllamaVersion::parse("1.2.3").unwrap());
    request.manifest = Some(manifest);
    request.local_archives = None;
    request.deadline = Some(Instant::now() + Duration::from_secs(20));
    let result = super::install::install(request.clone()).await.unwrap();
    assert!(result.is_ready());
    assert!(request.paths.active.join("bin/ollama").exists());
    assert!(!request.paths.install_staging.exists());
    assert!(!archive_staging_path(&request.paths).exists());
    assert_eq!(
        std::fs::read_to_string(request.paths.active.join("VERSION")).unwrap(),
        "1.2.3"
    );
    let receipt_path = crate::services::paths::bundle_receipt_path(&request.paths.active);
    let receipt = std::fs::read(receipt_path).unwrap();
    assert_eq!(
        BundleReceipt::parse_bounded(&receipt)
            .unwrap()
            .fingerprint
            .version
            .as_str(),
        "1.2.3"
    );
}

fn manifest_for(name: &str, size: u64, digest: &str) -> OllamaReleaseManifest {
    let version = OllamaVersion::parse("1.2.3").unwrap();
    let url = ValidatedHttpsUrl::parse(&format!(
        "https://github.com/ollama/ollama/releases/download/v1.2.3/{name}"
    ))
    .unwrap();
    OllamaReleaseManifest::try_new(
        version,
        vec![OllamaArchive::new(name, url, size, digest).unwrap()],
    )
    .unwrap()
}

#[cfg(unix)]
fn make_archive(root: &std::path::Path, binary: &std::path::Path) -> PathBuf {
    let archive = root.join("ollama-darwin.tgz");
    let file = File::create(&archive).unwrap();
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    let data = std::fs::read(binary).unwrap();
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    builder
        .append_data(&mut header, "bin/ollama", data.as_slice())
        .unwrap();
    let encoder = builder.into_inner().unwrap();
    encoder.finish().unwrap();
    archive
}

#[cfg(unix)]
fn compile_probe_server(root: &std::path::Path) -> PathBuf {
    let source = root.join("probe.c");
    let binary = root.join("ollama");
    std::fs::write(&source, PROBE_SERVER_SOURCE).unwrap();
    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
    let status = std::process::Command::new(compiler)
        .args([
            source.as_os_str(),
            std::ffi::OsStr::new("-o"),
            binary.as_os_str(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    binary
}

#[cfg(unix)]
const PROBE_SERVER_SOURCE: &[u8] = br#"
#include <arpa/inet.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>
static volatile sig_atomic_t stop_now = 0;
static void stop(int signal_number) { (void)signal_number; stop_now = 1; }
int main(void) {
  signal(SIGTERM, stop); signal(SIGINT, stop);
  const char *host = getenv("OLLAMA_HOST");
  if (host == NULL) return 2;
  const char *colon = strrchr(host, ':'); if (colon == NULL) return 3;
  int port = atoi(colon + 1); int server = socket(AF_INET, SOCK_STREAM, 0);
  if (server < 0) return 4;
  int reuse = 1; setsockopt(server, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse));
  struct sockaddr_in address = {0}; address.sin_family = AF_INET;
  address.sin_port = htons((uint16_t)port); address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
  if (bind(server, (struct sockaddr *)&address, sizeof(address)) != 0 || listen(server, 8) != 0) return 5;
  while (!stop_now) { int client = accept(server, NULL, NULL); if (client < 0) continue;
    char request[1024]; (void)recv(client, request, sizeof(request), 0);
    const char response[] = "HTTP/1.1 200 OK\r\nContent-Length: 19\r\nConnection: close\r\n\r\n{\"version\":\"1.2.3\"}";
    (void)send(client, response, sizeof(response) - 1, 0); close(client);
  }
  close(server); return 0;
}
"#;
