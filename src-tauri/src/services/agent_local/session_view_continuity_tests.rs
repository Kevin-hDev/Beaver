use super::session_view::continuity_capability;
use super::session_view_test_support::fixture_session;

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
