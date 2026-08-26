use super::session_view::continuity_capability;
use super::session_view_test_support::fixture_session;

#[test]
fn local_capability_requires_exact_live_user_and_tool_policies() {
    let mut session = fixture_session();
    session.provider = "ollama".into();
    session.model = "qwen3.5:4b".into();
    session.reasoning_mode = Some("auto".into());

    let capability = continuity_capability(&session).expect("validated Qwen capability");
    assert!(capability.local_available);

    session.reasoning_mode = Some("off".into());
    assert!(continuity_capability(&session).is_none());
}
