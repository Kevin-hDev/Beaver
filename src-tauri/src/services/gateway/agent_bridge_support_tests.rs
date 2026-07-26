use super::*;
use crate::models::GatewayConfig;
use crate::services::gateway::channels::InboundMessage;
use crate::services::gateway::types::ChannelKey;

fn inbound(chat_id: &str, thread_id: Option<&str>) -> InboundMessage {
    InboundMessage {
        channel_key: ChannelKey::new("slack", "work"),
        user_id: "U123".into(),
        content: "bonjour".into(),
        message_id: "171.2".into(),
        chat_id: chat_id.into(),
        thread_id: thread_id.map(str::to_string),
        is_group: true,
        mentions_bot: true,
    }
}

#[test]
fn conversation_key_is_stable_and_versioned() {
    let first = build_external_key(&inbound("C1", Some("170.1")));
    let second = build_external_key(&inbound("C1", Some("170.1")));

    assert_eq!(first, second);
    assert!(first.starts_with("gateway:v2:"));
    assert_eq!(first.len(), "gateway:v2:".len() + 64);
}

#[test]
fn conversation_key_separates_chat_and_thread() {
    let base = build_external_key(&inbound("C1", Some("170.1")));

    assert_ne!(base, build_external_key(&inbound("C2", Some("170.1"))));
    assert_ne!(base, build_external_key(&inbound("C1", Some("170.2"))));
    assert_ne!(base, build_external_key(&inbound("C1", None)));
}

#[test]
fn disabled_account_is_not_selected() {
    let mut config = GatewayConfig::default();
    let account = crate::models::ChannelAccountConfig {
        account_id: "work".into(),
        enabled: false,
        ..crate::models::ChannelAccountConfig::default()
    };
    config.channels.slack.push(account);

    assert!(find_account_config(&config, &inbound("C1", None)).is_none());
}
