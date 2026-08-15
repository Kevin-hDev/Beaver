use super::fingerprint::OllamaVersion;
use super::probe_http::fetch_version;
use super::probe_http::{parse_version_body, HttpProbeError};
use super::types::OllamaEndpoint;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[test]
fn exact_version_json_is_parsed_with_documented_v_prefix_normalization() {
    assert_eq!(
        parse_version_body(br#"{"version":"v1.2.3"}"#).expect("version"),
        OllamaVersion::parse("1.2.3").expect("version")
    );
}

#[test]
fn incomplete_or_ill_formed_json_is_invalid() {
    for body in [
        br#"{}"#.as_slice(),
        br#"not-json"#.as_slice(),
        br#"{"version":""}"#.as_slice(),
    ] {
        assert_eq!(parse_version_body(body), Err(HttpProbeError::Malformed));
    }
}

#[test]
fn malformed_semver_is_invalid() {
    assert_eq!(
        parse_version_body(br#"{"version":"latest"}"#),
        Err(HttpProbeError::Malformed)
    );
}

#[tokio::test]
async fn response_larger_than_four_kib_is_rejected_before_json_parse() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'x'; 4097]))
        .mount(&server)
        .await;
    let endpoint = endpoint_from_server(&server).await;
    let error = fetch_version(
        &endpoint,
        Instant::now() + Duration::from_secs(1),
        &CancellationToken::new(),
    )
    .await
    .expect_err("oversized response");
    assert_eq!(error, HttpProbeError::Oversized);
}

#[tokio::test]
async fn cancellation_is_observed_while_waiting_for_http() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(1)))
        .mount(&server)
        .await;
    let endpoint = endpoint_from_server(&server).await;
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert_eq!(
        fetch_version(
            &endpoint,
            Instant::now() + Duration::from_secs(1),
            &cancellation,
        )
        .await,
        Err(HttpProbeError::Cancelled)
    );
}

#[tokio::test]
async fn redirects_are_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("location", "/other")
                .set_body_string("redirect"),
        )
        .mount(&server)
        .await;
    let endpoint = endpoint_from_server(&server).await;
    assert_eq!(
        fetch_version(
            &endpoint,
            Instant::now() + Duration::from_secs(1),
            &CancellationToken::new(),
        )
        .await,
        Err(HttpProbeError::Redirect)
    );
}

async fn endpoint_from_server(server: &MockServer) -> OllamaEndpoint {
    OllamaEndpoint::try_from_http_url(&server.uri()).expect("loopback endpoint")
}
