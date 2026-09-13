use super::download::{
    download_once, download_once_with_timeouts, download_with_retries, percent, redirect_allowed,
    DownloadProgress, ASSET_TIMEOUT, CONNECT_TIMEOUT, IDLE_CHUNK_TIMEOUT, MAX_RESPONSE_HEADERS,
    MAX_RESPONSE_HEADER_BYTES,
};
use super::launch_args::PinnedRelease;
use super::temp_ownership::{OwnedTempRun, OWNER_MARKER};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

static NEXT: AtomicU64 = AtomicU64::new(1);
const RUN_ID: &str = "0123456789abcdef0123456789abcdef";

struct Fixture {
    root: PathBuf,
    run: Option<OwnedTempRun>,
}

impl Fixture {
    fn new(body: &[u8]) -> (Self, PinnedRelease) {
        let root = std::env::temp_dir().join(format!(
            "beaver-download-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let work = root.join(format!("beaver-install-{RUN_ID}"));
        fs::create_dir(&work).unwrap();
        fs::write(
            work.join(OWNER_MARKER),
            format!(r#"{{"schema":1,"runId":"{RUN_ID}"}}"#),
        )
        .unwrap();
        let run = OwnedTempRun::adopt(&root, &work, RUN_ID).unwrap();
        let version = "1.2.3".to_string();
        let app_asset_name = if cfg!(target_os = "macos") {
            "Beaver_1.2.3_aarch64.dmg"
        } else {
            "Beaver_1.2.3_x64-setup.exe"
        }
        .to_string();
        let release = PinnedRelease {
            version,
            app_asset_name,
            app_asset_size: body.len() as u64,
            app_asset_sha256: hex::encode(Sha256::digest(body)),
        };
        (
            Self {
                root,
                run: Some(run),
            },
            release,
        )
    }

    fn run(&self) -> &OwnedTempRun {
        self.run.as_ref().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.run.take();
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn server(response: Vec<u8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 2048];
        let read = stream.read(&mut request).unwrap();
        assert!(String::from_utf8_lossy(&request[..read]).contains("accept-encoding: identity"));
        stream.write_all(&response).unwrap();
    });
    format!("http://{address}/asset")
}

fn response(status: &str, headers: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes = format!("HTTP/1.1 {status}\r\nConnection: close\r\n{headers}\r\n").into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

fn sequence_server(responses: Vec<Vec<u8>>) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&count);
    std::thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 2048];
            let _read = stream.read(&mut request).unwrap();
            observed.fetch_add(1, Ordering::SeqCst);
            stream.write_all(&response).unwrap();
        }
    });
    (format!("http://{address}/asset"), count)
}

fn stalled_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 2048];
        let _read = stream.read(&mut request).unwrap();
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n")
            .unwrap();
        stream.flush().unwrap();
        std::thread::sleep(Duration::from_millis(200));
    });
    format!("http://{address}/asset")
}

#[tokio::test]
async fn downloads_valid_body_with_or_without_content_length() {
    for header in ["Content-Length: 4\r\n", ""] {
        let (fixture, release) = Fixture::new(b"test");
        let url = server(response("200 OK", header, b"test"));
        let path = download_once(
            reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            reqwest::Url::parse(&url).unwrap(),
            &release,
            fixture.run(),
            &CancellationToken::new(),
            |_| {},
        )
        .await
        .unwrap();
        assert_eq!(fs::read(path).unwrap(), b"test");
    }
}

#[tokio::test]
async fn retry_reuses_the_verified_asset_without_a_second_download() {
    let (fixture, release) = Fixture::new(b"test");
    let expected = fixture.run().path().join(&release.app_asset_name);
    fs::write(&expected, b"test").unwrap();

    let path = download_with_retries(
        reqwest::Client::new(),
        reqwest::Url::parse("http://127.0.0.1:1/unreachable").unwrap(),
        &release,
        fixture.run(),
    )
    .await
    .unwrap();

    assert_eq!(path, expected);
}

#[tokio::test]
async fn retry_replaces_a_corrupt_owned_asset() {
    let (fixture, release) = Fixture::new(b"test");
    let expected = fixture.run().path().join(&release.app_asset_name);
    fs::write(&expected, b"bad!").unwrap();
    let url = server(response("200 OK", "Content-Length: 4\r\n", b"test"));

    let path = download_with_retries(
        reqwest::Client::new(),
        reqwest::Url::parse(&url).unwrap(),
        &release,
        fixture.run(),
    )
    .await
    .unwrap();

    assert_eq!(path, expected);
    assert_eq!(fs::read(path).unwrap(), b"test");
}

#[tokio::test]
async fn rejects_status_length_truncation_overflow_and_hash_mismatch() {
    let cases = [
        response("404 Not Found", "Content-Length: 4\r\n", b"test"),
        response("200 OK", "Content-Length: 3\r\n", b"test"),
        response(
            "200 OK",
            "Content-Length: 4\r\nContent-Encoding: gzip\r\n",
            b"test",
        ),
        response("200 OK", "Content-Length: 4\r\n", b"tes"),
        response("200 OK", "", b"tests"),
    ];
    for wire in cases {
        let (fixture, release) = Fixture::new(b"test");
        let url = server(wire);
        assert!(download_once(
            reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            reqwest::Url::parse(&url).unwrap(),
            &release,
            fixture.run(),
            &CancellationToken::new(),
            |_| {},
        )
        .await
        .is_err());
        assert!(!fixture
            .run()
            .path()
            .join(format!("{}.part", release.app_asset_name))
            .exists());
    }

    let (fixture, mut release) = Fixture::new(b"test");
    release.app_asset_sha256 = "0".repeat(64);
    let url = server(response("200 OK", "Content-Length: 4\r\n", b"test"));
    assert!(download_once(
        reqwest::Client::new(),
        reqwest::Url::parse(&url).unwrap(),
        &release,
        fixture.run(),
        &CancellationToken::new(),
        |_| {},
    )
    .await
    .is_err());
}

#[tokio::test]
async fn cancellation_removes_the_partial_file() {
    let (fixture, release) = Fixture::new(b"test");
    let cancel = CancellationToken::new();
    cancel.cancel();
    let url = server(response("200 OK", "Content-Length: 4\r\n", b"test"));
    assert!(download_once(
        reqwest::Client::new(),
        reqwest::Url::parse(&url).unwrap(),
        &release,
        fixture.run(),
        &cancel,
        |_| {},
    )
    .await
    .is_err());
    assert!(!fixture
        .run()
        .path()
        .join(format!("{}.part", release.app_asset_name))
        .exists());
}

#[test]
fn redirect_and_percentage_contracts_are_closed() {
    assert!(redirect_allowed(
        &reqwest::Url::parse("https://release-assets.githubusercontent.com/file?x=1").unwrap()
    ));
    for denied in [
        "http://release-assets.githubusercontent.com/file",
        "https://user@release-assets.githubusercontent.com/file",
        "https://release-assets.githubusercontent.com:444/file",
        "https://example.com/file",
        "https://release-assets.githubusercontent.com/file#fragment",
    ] {
        assert!(
            !redirect_allowed(&reqwest::Url::parse(denied).unwrap()),
            "accepted {denied}"
        );
    }
    assert_eq!(
        percent(DownloadProgress {
            completed: 1,
            total: 0
        }),
        None
    );
    assert_eq!(
        percent(DownloadProgress {
            completed: 3,
            total: 2
        }),
        Some(100)
    );
    assert_eq!(CONNECT_TIMEOUT.as_secs(), 10);
    assert_eq!(IDLE_CHUNK_TIMEOUT.as_secs(), 30);
    assert_eq!(ASSET_TIMEOUT.as_secs(), 30 * 60);
    assert_eq!(MAX_RESPONSE_HEADERS, 64);
    assert_eq!(MAX_RESPONSE_HEADER_BYTES, 64 * 1024);
}

#[tokio::test]
async fn retries_only_server_failures_and_stops_after_three_attempts() {
    let (fixture, release) = Fixture::new(b"test");
    let wires = vec![
        response("500 Internal Server Error", "Content-Length: 0\r\n", b""),
        response("503 Service Unavailable", "Content-Length: 0\r\n", b""),
        response("200 OK", "Content-Length: 4\r\n", b"test"),
    ];
    let (url, count) = sequence_server(wires);
    let started = Instant::now();
    download_with_retries(
        reqwest::Client::new(),
        reqwest::Url::parse(&url).unwrap(),
        &release,
        fixture.run(),
    )
    .await
    .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 3);
    assert!(started.elapsed() >= Duration::from_secs(3));

    let (fixture, release) = Fixture::new(b"test");
    let (url, count) = sequence_server(vec![response(
        "400 Bad Request",
        "Content-Length: 0\r\n",
        b"",
    )]);
    assert!(download_with_retries(
        reqwest::Client::new(),
        reqwest::Url::parse(&url).unwrap(),
        &release,
        fixture.run(),
    )
    .await
    .is_err());
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn rejects_response_header_count_and_byte_overflow() {
    let many = (0..65)
        .map(|index| format!("X-Test-{index}: value\r\n"))
        .collect::<String>();
    let huge = format!("X-Huge: {}\r\n", "a".repeat(MAX_RESPONSE_HEADER_BYTES));
    for headers in [many, huge] {
        let (fixture, release) = Fixture::new(b"test");
        let url = server(response("200 OK", &headers, b"test"));
        assert!(download_once(
            reqwest::Client::new(),
            reqwest::Url::parse(&url).unwrap(),
            &release,
            fixture.run(),
            &CancellationToken::new(),
            |_| {},
        )
        .await
        .is_err());
    }
}

#[tokio::test]
async fn idle_and_total_timeouts_abort_and_clean_the_partial_file() {
    for (idle, total) in [
        (Duration::from_millis(10), Duration::from_secs(1)),
        (Duration::from_secs(1), Duration::from_millis(10)),
    ] {
        let (fixture, release) = Fixture::new(b"test");
        let url = stalled_server();
        assert!(download_once_with_timeouts(
            reqwest::Client::new(),
            reqwest::Url::parse(&url).unwrap(),
            &release,
            fixture.run(),
            idle,
            total,
        )
        .await
        .is_err());
        assert!(!fixture
            .run()
            .path()
            .join(format!("{}.part", release.app_asset_name))
            .exists());
    }
}
