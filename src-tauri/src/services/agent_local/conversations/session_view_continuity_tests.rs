use super::session_view::continuity_capability;
use super::session_view_test_support::fixture_session;
use super::types_session::PreserveReasoningSetting;

#[test]
fn local_capability_requires_exact_live_user_and_tool_policies() {
    let mut session = fixture_session();
    session.provider = "ollama".into();
    session.reasoning_mode = Some("auto".into());

    for model in ["qwen3.5:4b", "gemma4:e2b-it-q4_K_M"] {
        session.model = model.into();
        let capability = continuity_capability(&session).expect("validated local capability");
        assert!(capability.local_available);
    }

    session.reasoning_mode = Some("off".into());
    assert!(continuity_capability(&session).is_none());

    session.model = "deepseek-r1:latest".into();
    session.reasoning_mode = Some("auto".into());
    assert!(continuity_capability(&session).is_none());
}

#[test]
fn remote_stays_hidden_until_a_previous_response_fixture_exists() {
    let mut session = fixture_session();
    session.provider = "ollama".into();
    session.model = "qwen3.5:4b".into();
    session.reasoning_mode = Some("auto".into());
    let capability = continuity_capability(&session).expect("validated local capability");

    assert!(capability.local_available);
    assert!(!capability.remote_available);

    session.preserve_reasoning = PreserveReasoningSetting::Remote;
    assert_eq!(
        super::session_view::from_session(&session)
            .unwrap()
            .preserve_reasoning,
        PreserveReasoningSetting::Local
    );
}
