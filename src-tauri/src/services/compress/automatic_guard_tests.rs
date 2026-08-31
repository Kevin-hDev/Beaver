use super::*;

fn attempt(message: &str, model: &str) -> AutomaticCompressionAttempt {
    AutomaticCompressionAttempt {
        top_level_turn_id: "00000000-0000-4000-8000-000000000001".into(),
        last_message_id: message.into(),
        message_count: 2,
        last_checkpoint_message_id: None,
        provider_id: "ollama".into(),
        model_id: model.into(),
        context_window: 96_000,
        profile_id: "beaver".into(),
        profile_revision: 1,
        global_selection_revision: 1,
    }
}

#[test]
fn exact_snapshot_is_attempted_only_once() {
    let mut guard = AutomaticCompressionGuard::default();
    let current = attempt("00000000-0000-4000-8000-000000000002", "model");

    assert!(matches!(
        start(&mut guard, &current),
        StartDecision::Proceed
    ));
    assert!(matches!(
        start(&mut guard, &current),
        StartDecision::AlreadyAttempted
    ));
}

#[test]
fn three_distinct_failures_suspend_the_environment() {
    let mut guard = AutomaticCompressionGuard::default();
    for index in 2..=4 {
        let current = attempt(&format!("00000000-0000-4000-8000-{index:012}"), "model");
        assert!(matches!(
            start(&mut guard, &current),
            StartDecision::Proceed
        ));
        fail(&mut guard);
    }

    assert_eq!(guard.consecutive_failures, 3);
    assert!(guard.suspended);
}

#[test]
fn model_change_starts_a_new_failure_series() {
    let mut guard = AutomaticCompressionGuard::default();
    let old = attempt("00000000-0000-4000-8000-000000000002", "old");
    start(&mut guard, &old);
    fail(&mut guard);
    let changed = attempt("00000000-0000-4000-8000-000000000003", "new");

    assert!(matches!(
        start(&mut guard, &changed),
        StartDecision::Proceed
    ));
    assert_eq!(guard.consecutive_failures, 0);
    assert!(!guard.suspended);
}

#[test]
fn successful_result_below_threshold_clears_the_guard() {
    let guard = success_guard(
        Some(attempt("00000000-0000-4000-8000-000000000002", "model")),
        30_000,
        96_000,
        90,
    );

    assert!(guard.is_empty());
}

#[test]
fn successful_result_still_above_threshold_blocks_the_same_top_level_turn() {
    let original = attempt("00000000-0000-4000-8000-000000000002", "model");
    let mut guard = success_guard(Some(original.clone()), 90_000, 96_000, 90);
    let mut after_checkpoint = original;
    after_checkpoint.last_message_id = "00000000-0000-4000-8000-000000000099".into();
    after_checkpoint.message_count = 4;
    after_checkpoint.last_checkpoint_message_id =
        Some("00000000-0000-4000-8000-000000000098".into());

    assert!(matches!(
        start(&mut guard, &after_checkpoint),
        StartDecision::AlreadyAttempted
    ));
    assert!(same_successful_top_level(&guard, &after_checkpoint));
}

async fn stored_session() -> AgentSession {
    let fixture = super::super::snapshot_tests::session();
    let mut session = crate::services::agent_local::session_store::create_full(
        "automatic guard",
        &fixture.model,
        &fixture.provider,
        false,
        None,
    )
    .await
    .unwrap();
    session.messages = fixture.messages;
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    session
}

#[tokio::test]
async fn persisted_attempt_survives_reload_and_blocks_the_same_snapshot() {
    let session = stored_session().await;
    let profile = super::super::profile_resolve::resolve_for_session(&session).unwrap();
    let prepared = prepare(&session, &profile, 128_000, CompressionTrigger::Automatic)
        .await
        .unwrap()
        .expect("first attempt");
    let persisted = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();

    assert_eq!(
        persisted.automatic_compression_guard.last_attempt,
        prepared.attempt
    );
    assert!(
        prepare(&persisted, &profile, 128_000, CompressionTrigger::Automatic,)
            .await
            .unwrap()
            .is_none()
    );
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn explicit_compression_remains_available_without_clearing_guard_before_success() {
    let mut session = stored_session().await;
    let profile = super::super::profile_resolve::resolve_for_session(&session).unwrap();
    session.automatic_compression_guard = AutomaticCompressionGuard {
        last_attempt: Some(attempt(
            "00000000-0000-4000-8000-000000000002",
            &session.model,
        )),
        consecutive_failures: 3,
        suspended: true,
    };
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();

    let prepared = prepare(&session, &profile, 128_000, CompressionTrigger::Explicit)
        .await
        .unwrap()
        .expect("explicit compression remains available");

    assert!(prepared.session.automatic_compression_guard.suspended);
    assert!(prepared.attempt.is_none());
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_persisted_user_turn_rearms_automatic_compression() {
    let session = stored_session().await;
    let profile = super::super::profile_resolve::resolve_for_session(&session).unwrap();
    prepare(&session, &profile, 128_000, CompressionTrigger::Automatic)
        .await
        .unwrap()
        .expect("attempt persisted");
    let turn = crate::services::agent_local::types_message::AgentMessage::new_turn_id();
    let user = super::super::checkpoint_messages_tests::message(&turn, "user", "new request");
    let assistant =
        super::super::checkpoint_messages_tests::message(&turn, "assistant", "new response");

    crate::services::agent_local::session_store_messages::add_messages(
        &session.id,
        vec![user, assistant],
        0,
    )
    .await
    .unwrap();

    let reloaded = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert!(reloaded.automatic_compression_guard.is_empty());
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
