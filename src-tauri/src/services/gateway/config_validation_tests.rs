use super::*;

fn enabled_account() -> ChannelAccountConfig {
    ChannelAccountConfig {
        enabled: true,
        allowlist: vec!["user-1".to_string()],
        provider: "openai".to_string(),
        model: "gpt-test".to_string(),
        ..ChannelAccountConfig::default()
    }
}

#[test]
fn accepts_bounded_allowlisted_configuration() {
    let mut config = GatewayConfig::default();
    config.channels.telegram.push(enabled_account());
    assert!(validate(&config).is_ok());
}

#[test]
fn rejects_wildcard_empty_or_oversized_allowlists() {
    for users in [vec![], vec!["*".to_string()], vec!["u".to_string(); 101]] {
        let mut config = GatewayConfig::default();
        let mut account = enabled_account();
        account.allowlist = users;
        config.channels.telegram.push(account);
        assert!(validate(&config).is_err());
    }
}

#[test]
fn rejects_too_many_accounts_sessions_messages_and_rates() {
    let mut config = GatewayConfig::default();
    config.channels.slack = (0..17)
        .map(|index| ChannelAccountConfig {
            account_id: format!("account-{index}"),
            ..ChannelAccountConfig::default()
        })
        .collect();
    assert!(validate(&config).is_err());

    for mutate in [
        |cfg: &mut GatewayConfig| cfg.max_sessions = 1_001,
        |cfg: &mut GatewayConfig| cfg.message_max_chars = 12_001,
        |cfg: &mut GatewayConfig| cfg.rate_limits.global_per_minute = 10_001,
        |cfg: &mut GatewayConfig| cfg.audit.retention_days = 366,
    ] {
        let mut invalid = GatewayConfig::default();
        mutate(&mut invalid);
        assert!(validate(&invalid).is_err());
    }
}

#[test]
fn legacy_security_fields_are_ignored_and_disappear() {
    let value = serde_json::json!({
        "max_messages_per_session": 50,
        "security": {"allow_private_urls": true},
        "audit": {"enabled": true, "retention_days": 30, "redact_content": false}
    });
    let config: GatewayConfig = serde_json::from_value(value).unwrap();
    let serialized = serde_json::to_string(&config).unwrap();
    assert!(!serialized.contains("max_messages_per_session"));
    assert!(!serialized.contains("allow_private_urls"));
    assert!(!serialized.contains("redact_content"));
}

#[test]
fn rejects_interactive_only_oauth_providers() {
    for provider in ["xai-oauth", "moonshot-oauth"] {
        let mut config = GatewayConfig::default();
        let mut account = enabled_account();
        account.provider = provider.to_string();
        config.channels.telegram.push(account);
        assert!(validate(&config).is_err());
    }
}
