use super::*;

#[test]
fn slack_requires_both_tokens() {
    let missing_app = AccountTokens {
        token: None,
        bot_token: Some("xoxb-test".into()),
        app_token: None,
    };
    assert!(missing_app.validate_for("slack").is_err());

    let complete = AccountTokens {
        token: None,
        bot_token: Some("xoxb-test".into()),
        app_token: Some("xapp-test".into()),
    };
    assert!(complete.validate_for("slack").is_ok());
    assert_eq!(complete.vault_entries("slack", "work").unwrap().len(), 2);
}

#[test]
fn single_token_channels_reject_extra_secrets() {
    let credentials = AccountTokens {
        token: Some("test-token".into()),
        bot_token: Some("unexpected".into()),
        app_token: None,
    };
    assert!(credentials.validate_for("telegram").is_err());
}
