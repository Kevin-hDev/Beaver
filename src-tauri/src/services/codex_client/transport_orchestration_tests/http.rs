use super::{assert_http_capture, messages};
use crate::services::codex_client::test_transport::{CodexTransportScenario, HttpReply};
use crate::services::codex_client::{request, stream_silent};
use crate::services::llm::fast_mode::FastModeRequest;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn request_keeps_body_and_header_aligned_for_every_mode() {
    for (mode, model, expected_tier, expected_hint) in [
        (
            FastModeRequest::Fast,
            "gpt-5.6-sol",
            Some("priority"),
            "model=gpt-5.6-sol;tier=priority",
        ),
        (
            FastModeRequest::Standard,
            "gpt-5.6-sol",
            None,
            "model=gpt-5.6-sol",
        ),
        (
            FastModeRequest::Unsupported,
            "gpt-5.4-mini",
            None,
            "model=gpt-5.4-mini",
        ),
    ] {
        let scenario = CodexTransportScenario::start(Some(vec![HttpReply::Success]), None).await;
        let response = request::post_codex_stream(
            model,
            &messages(),
            &[],
            None,
            None,
            mode,
            &CancellationToken::new(),
        )
        .await
        .expect("loopback POST succeeds");
        drop(response);

        let captures = scenario.http_captures();
        assert_eq!(captures.len(), 1);
        assert_http_capture(&captures[0], model, expected_tier, expected_hint);
    }
}

#[tokio::test]
async fn unauthorized_refresh_reuses_the_exact_fast_pair_once() {
    let scenario = CodexTransportScenario::start(
        Some(vec![HttpReply::Unauthorized, HttpReply::Success]),
        None,
    )
    .await;
    let response = request::post_codex_stream(
        "gpt-5.6-sol",
        &messages(),
        &[],
        None,
        None,
        FastModeRequest::Fast,
        &CancellationToken::new(),
    )
    .await
    .expect("refresh succeeds");
    drop(response);

    let captures = scenario.http_captures();
    assert_eq!(captures.len(), 2);
    assert_eq!(scenario.refresh_count(), 1);
    assert_eq!(captures[0].body, captures[1].body);
    assert_eq!(captures[0].routing_hint, captures[1].routing_hint);
    for capture in &captures {
        assert_http_capture(
            capture,
            "gpt-5.6-sol",
            Some("priority"),
            "model=gpt-5.6-sol;tier=priority",
        );
    }
}

#[tokio::test]
async fn silent_compression_keeps_the_explicit_fast_capture() {
    let session = crate::services::agent_local::session_store::create_with_project_and_fast_mode(
        "Codex silent capture",
        "gpt-5.6-sol",
        "codex-oauth",
        None,
        false,
    )
    .await
    .expect("create Standard session");
    let scenario = CodexTransportScenario::start(Some(vec![HttpReply::Success]), None).await;
    let result = stream_silent::collect_chat_silent_for_compression(
        "gpt-5.6-sol",
        &messages(),
        &[],
        None,
        FastModeRequest::Fast,
        Some(64),
        Some(&session.id),
        CancellationToken::new(),
        None,
    )
    .await;
    let captures = scenario.http_captures();
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    result.expect("silent compression succeeds");
    assert_eq!(captures.len(), 1);
    assert_http_capture(
        &captures[0],
        "gpt-5.6-sol",
        Some("priority"),
        "model=gpt-5.6-sol;tier=priority",
    );
}
