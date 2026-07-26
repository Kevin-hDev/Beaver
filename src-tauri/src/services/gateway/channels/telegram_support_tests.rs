use super::*;
use crate::services::gateway::channels::OutboundMessage;

fn topic_update() -> TgUpdate {
    TgUpdate {
        update_id: 1,
        message: Some(TgMessage {
            message_id: 9,
            message_thread_id: Some(77),
            from: Some(TgUser {
                id: 4,
                username: None,
            }),
            chat: TgChat {
                id: 3,
                chat_type: "supergroup".into(),
            },
            text: Some("bonjour".into()),
            entities: None,
        }),
    }
}

#[test]
fn topic_is_preserved_in_both_directions() {
    let key = ChannelKey::new("telegram", "main");
    let inbound = TelegramAdapter::to_inbound(&topic_update(), &key, false, "bot").unwrap();
    assert_eq!(inbound.thread_id.as_deref(), Some("77"));

    let body = TelegramAdapter::send_body(&OutboundMessage {
        chat_id: "3".into(),
        thread_id: inbound.thread_id,
        content: "réponse".into(),
        reply_to: None,
    });
    assert_eq!(body["message_thread_id"], "77");
}
