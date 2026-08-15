use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::net::TcpListener;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use super::error::OllamaErrorCode;
use super::path_identity::{NativePathIdentityResolver, PathIdentityResolver};
use super::port::OllamaPortAllocator;
use super::probe::{OllamaTargetProbe, OwnedOllamaTargetProbe, PreparedBundle, TargetValidation};
use super::probe_ownership::{wait_for_owned_endpoint, EndpointWaitResult};
use super::spawn_profile::OllamaSpawnProfile;
use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity};
use crate::services::paths::ollama_paths;

const DEAD_SCRIPT: &[u8] = b"#!/bin/sh\nexit 0\n";
#[cfg(unix)]
const SERVING_SOURCE: &[u8] = br#"
#define _POSIX_C_SOURCE 200809L
#define _DARWIN_C_SOURCE
#include <arpa/inet.h>
#include <errno.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <unistd.h>

static volatile sig_atomic_t stopping = 0;

static void request_stop(int signal_number) {
    (void)signal_number;
    stopping = 1;
}

static void record_event(const char *label) {
    const char *path = getenv("PROBE_EVENTS_FILE");
    if (path == NULL) {
        return;
    }
    struct timeval now;
    gettimeofday(&now, NULL);
    FILE *events = fopen(path, "a");
    if (events != NULL) {
        fprintf(events, "%ld %s %lld\n", (long)getpid(), label,
                (long long)now.tv_sec * 1000000LL + now.tv_usec);
        fclose(events);
    }
}

int main(void) {
    struct sigaction action;
    memset(&action, 0, sizeof(action));
    action.sa_handler = request_stop;
    sigemptyset(&action.sa_mask);
    sigaction(SIGTERM, &action, NULL);
    sigaction(SIGINT, &action, NULL);
    record_event("start");

    const char *host = getenv("OLLAMA_HOST");
    const char *separator = host == NULL ? NULL : strrchr(host, ':');
    if (separator == NULL) {
        return 2;
    }
    int port = atoi(separator + 1);
    int server = socket(AF_INET, SOCK_STREAM, 0);
    if (server < 0) {
        return 3;
    }
    int reuse = 1;
    setsockopt(server, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse));
    struct sockaddr_in address;
    memset(&address, 0, sizeof(address));
    address.sin_family = AF_INET;
    address.sin_port = htons((uint16_t)port);
    address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    if (bind(server, (struct sockaddr *)&address, sizeof(address)) != 0 ||
        listen(server, 8) != 0) {
        close(server);
        return 4;
    }
    while (!stopping) {
        int client = accept(server, NULL, NULL);
        if (client < 0) {
            if (errno == EINTR) {
                break;
            }
            continue;
        }
        char request[4096];
        (void)recv(client, request, sizeof(request), 0);
        const char body[] = "{\"version\":\"1.2.3\"}";
        char response[256];
        int length = snprintf(response, sizeof(response),
            "HTTP/1.1 200 OK\r\nContent-Length: %zu\r\nConnection: close\r\n\r\n%s",
            sizeof(body) - 1, body);
        if (length > 0) {
            (void)send(client, response, (size_t)length, 0);
        }
        close(client);
    }
    close(server);
    record_event("stop");
    return 0;
}
"#;

struct Fixture {
    _root: tempfile::TempDir,
    profile: OllamaSpawnProfile,
    target: PreparedBundle,
    events_file: PathBuf,
}

fn fixture() -> Fixture {
    fixture_with_script(DEAD_SCRIPT)
}

#[cfg(unix)]
fn serving_fixture() -> Fixture {
    fixture_with_native_server()
}

fn fixture_with_script(script: &[u8]) -> Fixture {
    let (root, root_path, executable, cwd, events_file) = fixture_paths();
    std::fs::write(&executable, script).expect("probe executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
            .expect("executable permissions");
    }
    finish_fixture(root, root_path, executable, cwd, events_file)
}

#[cfg(unix)]
fn fixture_with_native_server() -> Fixture {
    let (root, root_path, executable, cwd, events_file) = fixture_paths();
    let source = root_path.join("probe-server.c");
    std::fs::write(&source, SERVING_SOURCE).expect("probe server source");
    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
    let status = Command::new(compiler)
        .args([
            source.as_os_str(),
            std::ffi::OsStr::new("-o"),
            executable.as_os_str(),
        ])
        .status()
        .expect("compile probe server");
    assert!(status.success(), "probe server compiler failed");
    finish_fixture(root, root_path, executable, cwd, events_file)
}

fn fixture_paths() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf, PathBuf) {
    let root = tempfile::tempdir().expect("temporary bundle root");
    let root_path = dunce::canonicalize(root.path()).expect("canonical bundle root");
    let paths = ollama_paths(&root_path);
    let executable = paths.active.join("bin").join("ollama");
    std::fs::create_dir_all(executable.parent().expect("bin parent")).expect("bin directory");
    let cwd = root_path.join("cwd");
    std::fs::create_dir_all(&cwd).expect("working directory");
    let events_file = root_path.join("probe-events.log");
    (root, root_path, executable, cwd, events_file)
}

fn finish_fixture(
    root: tempfile::TempDir,
    root_path: PathBuf,
    executable: PathBuf,
    cwd: PathBuf,
    events_file: PathBuf,
) -> Fixture {
    let paths = ollama_paths(&root_path);
    let resolver = NativePathIdentityResolver;
    let path = std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into());
    let profile = OllamaSpawnProfile::resolve_probe(
        &paths,
        vec![
            ("HOME".into(), root_path.as_os_str().into()),
            ("PATH".into(), path),
            ("PROBE_EVENTS_FILE".into(), events_file.as_os_str().into()),
        ],
        &cwd,
        &resolver,
    )
    .expect("probe profile");
    let root_directory = resolver
        .canonical_directory(&paths.active)
        .expect("bundle directory");
    let fingerprint = BundleFingerprint {
        version: OllamaVersion::parse("1.2.3").expect("version"),
        executable_sha256: digest_for_file(&executable),
    };
    Fixture {
        _root: root,
        target: PreparedBundle {
            root: root_directory,
            executable: profile.executable().clone(),
            fingerprint,
        },
        profile,
        events_file,
    }
}

fn digest_for_file(path: &Path) -> Sha256Digest {
    let bytes = std::fs::read(path).expect("file bytes");
    Sha256Digest::from_hex(&hex::encode(Sha256::digest(bytes))).expect("digest")
}

fn assert_invalid(result: TargetValidation) {
    assert!(matches!(
        result,
        TargetValidation::InvalidTarget {
            code: OllamaErrorCode::OllamaBundleInvalid
        }
    ));
}

#[tokio::test]
async fn endpoint_ownership_wait_observes_an_expired_deadline() {
    let endpoint = super::types::OllamaEndpoint::loopback(NonZeroU16::new(1).expect("port"));
    let identity = OwnedProcessIdentity {
        pid: 1,
        native_scope: 1,
        native_start_time: 1,
        executable: 1,
    };
    let result = wait_for_owned_endpoint(
        &endpoint,
        identity,
        Instant::now() - Duration::from_millis(1),
        &CancellationToken::new(),
    )
    .await;
    assert_eq!(result, EndpointWaitResult::Deadline);
}

#[test]
fn missing_executable_is_classified_as_invalid_target() {
    let fixture = fixture();
    std::fs::remove_file(fixture.profile.executable().path()).expect("remove executable");
    let result = tokio::runtime::Runtime::new().expect("runtime").block_on(
        OwnedOllamaTargetProbe::new(
            AlwaysUnavailable::default(),
            Instant::now() + Duration::from_secs(1),
        )
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new()),
    );
    assert_invalid(result);
}

#[tokio::test]
async fn invalid_permissions_are_classified_before_spawn() {
    let fixture = fixture();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            fixture.profile.executable().path(),
            std::fs::Permissions::from_mode(0o644),
        )
        .expect("remove execute permission");
    }
    let result = OwnedOllamaTargetProbe::new(
        AlwaysUnavailable::default(),
        Instant::now() + Duration::from_secs(1),
    )
    .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
    .await;
    assert_invalid(result);
}

#[tokio::test]
async fn changed_executable_hash_is_classified_as_invalid() {
    let fixture = fixture();
    std::fs::write(fixture.profile.executable().path(), b"changed").expect("change executable");
    let result = OwnedOllamaTargetProbe::new(
        AlwaysUnavailable::default(),
        Instant::now() + Duration::from_secs(1),
    )
    .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
    .await;
    assert_invalid(result);
}

#[tokio::test]
async fn malformed_executable_format_is_rejected_before_port_allocation() {
    let fixture = fixture();
    std::fs::write(fixture.profile.executable().path(), b"not an executable")
        .expect("change executable format");
    let allocator = AlwaysUnavailable::default();
    let calls = Arc::clone(&allocator.calls);
    let result = OwnedOllamaTargetProbe::new(allocator, Instant::now() + Duration::from_secs(1))
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
        .await;
    assert_invalid(result);
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn three_unavailable_ports_are_deferred_without_spawning() {
    let fixture = fixture();
    let allocator = AlwaysUnavailable::default();
    let calls = Arc::clone(&allocator.calls);
    let result = OwnedOllamaTargetProbe::new(allocator, Instant::now() + Duration::from_secs(1))
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
        .await;
    assert!(matches!(
        result,
        TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaValidationDeferred
        }
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn launched_then_dead_child_is_deferred_and_models_are_cleaned() {
    let fixture = fixture();
    let models_path = fixture.profile.models_directory().path().to_path_buf();
    let result = OwnedOllamaTargetProbe::new(SinglePort, Instant::now() + Duration::from_secs(1))
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
        .await;
    assert!(matches!(result, TargetValidation::Deferred { .. }));
    assert!(!models_path.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn gated_child_owns_endpoint_serves_version_and_is_reaped() {
    let fixture = serving_fixture();
    let models_path = fixture.profile.models_directory().path().to_path_buf();
    let result = OwnedOllamaTargetProbe::new(SinglePort, Instant::now() + Duration::from_secs(5))
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
        .await;
    assert!(
        matches!(result, TargetValidation::Valid { .. }),
        "{result:?} events={:?}",
        std::fs::read_to_string(&fixture.events_file)
    );
    let events = read_events(&fixture.events_file);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].1, "start");
    assert_eq!(events[1].1, "stop");
    assert!(!OwnedProcess::process_exists(events[0].0));
    assert!(!models_path.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn concurrent_validate_calls_never_overlap_owned_children() {
    let fixture = serving_fixture();
    let probe = OwnedOllamaTargetProbe::new(SinglePort, Instant::now() + Duration::from_secs(8));
    let cancellation = CancellationToken::new();
    let first = probe.validate(&fixture.target, &fixture.profile, &cancellation);
    let second = probe.validate(&fixture.target, &fixture.profile, &cancellation);
    let (first, second) = tokio::join!(first, second);
    assert!(
        matches!(first, TargetValidation::Valid { .. }),
        "{first:?} events={:?}",
        std::fs::read_to_string(&fixture.events_file)
    );
    assert!(
        matches!(second, TargetValidation::Valid { .. }),
        "{second:?} events={:?}",
        std::fs::read_to_string(&fixture.events_file)
    );
    let events = read_events(&fixture.events_file);
    assert_eq!(events.len(), 4);
    assert_eq!(
        events
            .iter()
            .map(|event| event.1.as_str())
            .collect::<Vec<_>>(),
        ["start", "stop", "start", "stop"]
    );
    assert!(events[1].2 <= events[2].2);
}

#[tokio::test]
async fn cancellation_is_deferred_with_a_stable_code() {
    let fixture = fixture();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let result = OwnedOllamaTargetProbe::new(
        AlwaysUnavailable::default(),
        Instant::now() + Duration::from_secs(1),
    )
    .validate(&fixture.target, &fixture.profile, &cancellation)
    .await;
    assert!(matches!(
        result,
        TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaOperationCancelled
        }
    ));
}

#[cfg(unix)]
fn read_events(path: &Path) -> Vec<(u32, String, u128)> {
    std::fs::read_to_string(path)
        .expect("probe events")
        .lines()
        .map(|line| {
            let mut fields = line.split_whitespace();
            (
                fields.next().expect("pid").parse().expect("pid number"),
                fields.next().expect("event").to_owned(),
                fields
                    .next()
                    .expect("timestamp")
                    .parse()
                    .expect("timestamp number"),
            )
        })
        .collect()
}

#[derive(Default)]
struct AlwaysUnavailable {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl OllamaPortAllocator for AlwaysUnavailable {
    fn allocate_loopback(
        &self,
        _excluded: &[u16],
    ) -> Result<super::types::OllamaEndpoint, OllamaErrorCode> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Err(OllamaErrorCode::OllamaValidationDeferred)
    }

    async fn detect_external(
        &self,
    ) -> Result<Option<super::types::OllamaEndpoint>, OllamaErrorCode> {
        Ok(None)
    }
}

struct SinglePort;

#[async_trait]
impl OllamaPortAllocator for SinglePort {
    fn allocate_loopback(
        &self,
        excluded: &[u16],
    ) -> Result<super::types::OllamaEndpoint, OllamaErrorCode> {
        for _ in 0..4 {
            let listener = TcpListener::bind(("127.0.0.1", 0))
                .map_err(|_| OllamaErrorCode::OllamaValidationDeferred)?;
            let port = listener
                .local_addr()
                .map_err(|_| OllamaErrorCode::OllamaValidationDeferred)?
                .port();
            if !excluded.contains(&port) {
                return Ok(super::types::OllamaEndpoint::loopback(
                    NonZeroU16::new(port).expect("port"),
                ));
            }
        }
        Err(OllamaErrorCode::OllamaValidationDeferred)
    }

    async fn detect_external(
        &self,
    ) -> Result<Option<super::types::OllamaEndpoint>, OllamaErrorCode> {
        Ok(None)
    }
}

use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
