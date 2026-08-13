use super::{
    local_site_candidates::LocalSiteCandidate, local_site_probe::probe_candidate,
    local_site_types::LocalSiteProtocol,
};
#[cfg(target_os = "windows")]
use crate::services::app_log::should_emit;
#[cfg(target_os = "windows")]
use log::{Log, Metadata, Record};
#[cfg(target_os = "windows")]
use std::{
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
#[cfg(target_os = "windows")]
use tokio::{io::AsyncWriteExt, net::TcpListener};
#[cfg(target_os = "windows")]
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[cfg(target_os = "windows")]
const TLS_CHILD_ENV: &str = "BEAVER_LOCAL_TLS_PROBE_CHILD";
#[cfg(target_os = "windows")]
const WINDOWS_VERIFIER_TARGET: &str = "rustls_platform_verifier::verification::windows";
#[cfg(target_os = "windows")]
static VERIFIER_RECORDS: AtomicUsize = AtomicUsize::new(0);
#[cfg(target_os = "windows")]
static EMITTED_VERIFIER_RECORDS: AtomicUsize = AtomicUsize::new(0);

#[cfg(target_os = "windows")]
struct ProbeLogger;

#[cfg(target_os = "windows")]
impl Log for ProbeLogger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if record.target() != WINDOWS_VERIFIER_TARGET {
            return;
        }
        VERIFIER_RECORDS.fetch_add(1, Ordering::Relaxed);
        if should_emit(record.metadata()) {
            EMITTED_VERIFIER_RECORDS.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn flush(&self) {}
}

#[cfg(target_os = "windows")]
static PROBE_LOGGER: ProbeLogger = ProbeLogger;

#[tokio::test]
async fn probes_a_bounded_local_html_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/html")
                .set_body_string("<html><title>Projet local</title><body>ok</body></html>"),
        )
        .mount(&server)
        .await;

    let site = probe_candidate(LocalSiteCandidate {
        port: server.address().port(),
        ipv4: true,
        ipv6: false,
    })
    .await
    .unwrap();

    assert_eq!(site.title, "Projet local");
    assert_eq!(site.protocol, LocalSiteProtocol::Http);
    assert_eq!(site.url, format!("http://localhost:{}/", site.port));
}

#[tokio::test]
async fn ignores_non_html_responses() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string("{\"status\":\"ok\"}"),
        )
        .mount(&server)
        .await;

    let result = probe_candidate(LocalSiteCandidate {
        port: server.address().port(),
        ipv4: true,
        ipv6: false,
    })
    .await;
    assert!(result.is_err());
}

#[cfg(target_os = "windows")]
#[test]
fn self_signed_tls_probe_is_rejected_without_emitting_verifier_error() {
    if std::env::var_os(TLS_CHILD_ENV).is_some() {
        run_tls_probe_child();
        return;
    }

    let test_name = "services::browser::local_site_probe_tests::\
        self_signed_tls_probe_is_rejected_without_emitting_verifier_error";
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(TLS_CHILD_ENV, "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            panic!("the isolated TLS probe test exceeded 10 seconds");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let output = child.wait_with_output().unwrap();

    assert!(
        status.success(),
        "isolated TLS probe failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(target_os = "windows")]
fn run_tls_probe_child() {
    log::set_logger(&PROBE_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (port, server) = self_signed_tls_server().await;
            let result = probe_candidate(LocalSiteCandidate {
                port,
                ipv4: true,
                ipv6: false,
            })
            .await;
            server.await.unwrap();

            assert!(
                result.is_err(),
                "an untrusted local certificate was accepted"
            );
            assert!(
                VERIFIER_RECORDS.load(Ordering::Relaxed) > 0,
                "the real Windows verifier path was not exercised"
            );
            assert_eq!(
                EMITTED_VERIFIER_RECORDS.load(Ordering::Relaxed),
                0,
                "the expected verifier error escaped the local probe scope"
            );
        });
}

#[cfg(target_os = "windows")]
async fn self_signed_tls_server() -> (u16, tokio::task::JoinHandle<()>) {
    let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let key = tokio_rustls::rustls::pki_types::PrivatePkcs8KeyDer::from(
        certified.signing_key.serialize_der(),
    );
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certified.cert.der().clone()], key.into())
        .unwrap();
    let acceptor = TlsAcceptor::from(std::sync::Arc::new(config));
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        for _ in 0..2 {
            let (stream, _) = tokio::time::timeout(Duration::from_secs(3), listener.accept())
                .await
                .expect("the local probe did not connect")
                .unwrap();
            if let Ok(Ok(mut tls)) =
                tokio::time::timeout(Duration::from_secs(2), acceptor.accept(stream)).await
            {
                let _ = tls
                    .write_all(b"HTTP/1.1 200 OK\r\ncontent-type: text/html\r\n\r\n<html></html>")
                    .await;
            }
        }
    });
    (port, server)
}
