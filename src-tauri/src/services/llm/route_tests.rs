use super::*;

#[test]
fn api_and_oauth_routes_stay_separate() {
    let xai_api = resolve("xai").unwrap();
    let xai_oauth = resolve("xai-oauth").unwrap();
    let moonshot_api = resolve("moonshot").unwrap();
    let moonshot_oauth = resolve("moonshot-oauth").unwrap();

    assert_eq!(xai_api.base_url, "https://api.x.ai/v1");
    assert_eq!(xai_oauth.base_url, "https://api.x.ai/v1");
    assert_eq!(xai_api.chat_provider_id, "xai");
    assert_eq!(xai_oauth.chat_provider_id, "xai-oauth");
    assert!(!xai_api.is_oauth());
    assert!(xai_oauth.is_oauth());

    assert_eq!(moonshot_api.base_url, "https://api.moonshot.ai/v1");
    assert_eq!(moonshot_oauth.base_url, "https://api.kimi.com/coding/v1");
    assert_eq!(moonshot_api.chat_provider_id, "moonshot");
    assert_eq!(moonshot_oauth.chat_provider_id, "moonshot-oauth");
    assert!(!moonshot_api.is_oauth());
    assert!(moonshot_oauth.is_oauth());
}

#[test]
fn oauth_routes_are_interactive_only() {
    assert!(is_interactive_only("xai-oauth"));
    assert!(is_interactive_only("moonshot-oauth"));
    assert!(!is_interactive_only("xai"));
}

#[test]
fn interactive_oauth_is_refused_for_background_and_unknown_requests() {
    let oauth = resolve("xai-oauth").unwrap();
    let api = resolve("xai").unwrap();

    assert!(oauth.permits(RequestPurpose::ManualChat));
    assert!(oauth.permits(RequestPurpose::AccountMetadata));
    assert!(!oauth.permits(RequestPurpose::Automation));
    assert!(!oauth.permits(RequestPurpose::ExternalChannel));
    assert!(!oauth.permits(RequestPurpose::Unknown));
    assert!(api.permits(RequestPurpose::Automation));
}

#[test]
fn only_first_401_refreshes_and_second_401_invalidates() {
    assert_eq!(oauth_401_action(401, false), OAuth401Action::Refresh);
    assert_eq!(oauth_401_action(401, true), OAuth401Action::Invalidate);
    for status in [402, 403, 429] {
        assert_eq!(oauth_401_action(status, false), OAuth401Action::None);
        assert_eq!(oauth_401_action(status, true), OAuth401Action::None);
    }
}
