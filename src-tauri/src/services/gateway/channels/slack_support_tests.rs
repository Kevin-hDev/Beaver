use super::*;
use crate::services::gateway::channels::OutboundMessage;

fn event(channel: &str, text: &str, ts: &str, thread_ts: Option<&str>) -> SlackEvent {
    SlackEvent {
        event_type: "message".into(),
        text: Some(text.into()),
        user: Some("U1".into()),
        channel: Some(channel.into()),
        ts: Some(ts.into()),
        thread_ts: thread_ts.map(str::to_string),
        bot_id: None,
    }
}

#[test]
fn group_requires_bot_mention_when_enabled() {
    let key = ChannelKey::new("slack", "work");
    assert!(
        SlackAdapter::to_inbound(&event("C1", "bonjour", "10.1", None), &key, true, "B1").is_none()
    );

    let inbound = SlackAdapter::to_inbound(
        &event("C1", "bonjour <@B1>", "10.1", None),
        &key,
        true,
        "B1",
    )
    .unwrap();
    assert_eq!(inbound.message_id, "10.1");
    assert_eq!(inbound.thread_id.as_deref(), Some("10.1"));
    assert!(inbound.mentions_bot);
}

#[test]
fn direct_message_does_not_require_mention() {
    let key = ChannelKey::new("slack", "work");
    let inbound =
        SlackAdapter::to_inbound(&event("D1", "bonjour", "10.1", None), &key, true, "B1").unwrap();
    assert!(!inbound.is_group);
}

#[test]
fn reply_keeps_actual_message_and_root_thread() {
    let key = ChannelKey::new("slack", "work");
    let inbound = SlackAdapter::to_inbound(
        &event("C1", "<@B1> suite", "10.2", Some("10.1")),
        &key,
        true,
        "B1",
    )
    .unwrap();
    assert_eq!(inbound.message_id, "10.2");
    assert_eq!(inbound.thread_id.as_deref(), Some("10.1"));
}

#[test]
fn outbound_body_contains_thread_root() {
    let body = SlackAdapter::post_body(&OutboundMessage {
        chat_id: "C1".into(),
        thread_id: Some("10.1".into()),
        content: "réponse".into(),
        reply_to: Some("10.2".into()),
    });
    assert_eq!(body["thread_ts"], "10.1");
}
