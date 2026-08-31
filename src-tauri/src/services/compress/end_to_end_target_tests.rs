use super::super::checkpoint_candidate;
use super::super::checkpoint_target::checkpoint_target;
use super::super::profile_resolve::resolve_from_document;
use super::super::profile_store_document::CompressionProfileDocument;
use super::super::profile_types::{CompressionTrigger, CompressionWindowBand};
use crate::services::agent_local::types_message::AgentMessageKind;

#[tokio::test]
async fn real_candidate_obeys_the_96k_and_258k_beaver_targets() {
    let session = super::target_support::stored_session().await;
    let document = CompressionProfileDocument::default();

    for (window, before, band, acceptance) in [
        (96_000, 96_000, CompressionWindowBand::Compact, 32_000),
        (258_000, 258_000, CompressionWindowBand::Large, 40_000),
    ] {
        let captured = super::target_support::snapshot(
            &session,
            &document,
            window,
            before,
            12_000,
            CompressionTrigger::Explicit,
        );
        assert_eq!(captured.system_head_tokens, 12_000);
        let candidate =
            checkpoint_candidate::build(&captured, Some(&super::support::summary()), &[])
                .await
                .unwrap();
        assert_eq!(
            candidate.report.target_tokens,
            Some(checkpoint_target(before, 12_000, band))
        );
        assert!(candidate.after_tokens <= acceptance);
        assert!(candidate
            .persisted_messages
            .iter()
            .filter(|message| message.message_kind.is_none())
            .all(|message| session
                .messages
                .iter()
                .any(|source| { source.id == message.id && source.content == message.content })));
        let checkpoint = candidate
            .persisted_messages
            .iter()
            .find(|message| message.message_kind == Some(AgentMessageKind::CompressionCheckpoint))
            .unwrap();
        let body: serde_json::Value = serde_json::from_str(&checkpoint.content).unwrap();
        assert_eq!(body["metadata"]["before_tokens"], before);
        assert_eq!(body["metadata"]["after_tokens"], candidate.after_tokens);
        for section in super::super::summary_contract::required_sections() {
            assert!(body["summary"].as_str().unwrap().contains(section));
        }
    }

    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn small_windows_reduce_the_summary_limit_without_changing_the_profile() {
    let session = super::target_support::stored_session().await;
    let mut document = CompressionProfileDocument::default();
    document.profiles[0].allow_under_64k = true;
    let original_limit = document.profiles[0].under_64k.summary_max_tokens;

    for (window, before, head, expected_limit) in
        [(16_000, 16_000, 4_000, 2_000), (8_192, 8_192, 3_000, 1_038)]
    {
        let captured = super::target_support::snapshot(
            &session,
            &document,
            window,
            before,
            head,
            CompressionTrigger::Explicit,
        );
        let collector = super::target_support::RecordingCollector::new();
        let summary = super::super::orchestrator_summary::generate(&captured, &collector)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(collector.calls(), 1);
        assert_eq!(collector.limits(), [expected_limit]);
        let candidate = checkpoint_candidate::build(&captured, Some(&summary), &[])
            .await
            .unwrap();
        assert!(
            candidate.after_tokens
                <= checkpoint_target(before, head, CompressionWindowBand::Under64K)
        );
    }
    assert_eq!(
        document.profiles[0].under_64k.summary_max_tokens,
        original_limit
    );

    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_511_token_summary_refusal_skips_network_and_counts_the_automatic_failure() {
    let session = super::target_support::stored_session().await;
    let document = CompressionProfileDocument::default();
    let profile = resolve_from_document(None, &document).unwrap();
    let prepared = super::super::automatic_guard::prepare(
        &session,
        &profile,
        96_000,
        CompressionTrigger::Automatic,
    )
    .await
    .unwrap()
    .unwrap();
    let attempt = prepared.attempt.unwrap();
    let head = 12_000;
    let captured = super::target_support::snapshot(
        &prepared.session,
        &document,
        96_000,
        head + 2_555,
        head,
        CompressionTrigger::Automatic,
    );
    let before = serde_json::to_vec(&prepared.session.messages).unwrap();
    let collector = super::target_support::RecordingCollector::new();

    assert_eq!(
        super::super::orchestrator_summary::generate(&captured, &collector).await,
        Err(super::super::checkpoint_transaction::CompressionError::CapacityExceeded)
    );
    assert_eq!(collector.calls(), 0);
    super::super::automatic_guard::record_failure(&session.id, &attempt).await;
    let reloaded = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert_eq!(reloaded.automatic_compression_guard.consecutive_failures, 1);
    assert_eq!(serde_json::to_vec(&reloaded.messages).unwrap(), before);
    assert_eq!(reloaded.compression_count, session.compression_count);

    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
