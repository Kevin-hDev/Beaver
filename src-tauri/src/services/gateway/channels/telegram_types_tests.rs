use super::*;

#[test]
fn group_detection() {
    let group = TgChat {
        id: 1,
        chat_type: "group".into(),
    };
    let direct = TgChat {
        id: 2,
        chat_type: "private".into(),
    };
    assert!(group.is_group());
    assert!(!direct.is_group());
}

#[test]
fn bot_mention_detection() {
    let msg = message("@MyBot hello", 0);
    assert!(msg.has_bot_mention("MyBot"));
    assert!(msg.has_bot_mention("mybot"));
    assert!(!msg.has_bot_mention("OtherBot"));
}

#[test]
fn bot_mention_detection_uses_utf16_offsets() {
    assert!(message("😀 @MyBot bonjour", 3).has_bot_mention("mybot"));
}

#[test]
fn excessive_entities_are_rejected_during_deserialization() {
    let entities = vec![serde_json::json!({"type":"mention","offset":0,"length":1}); 101];
    let value = serde_json::json!({
        "message_id": 1,
        "from": null,
        "chat": {"id": 1, "type": "group"},
        "text": "x",
        "entities": entities,
    });
    assert!(serde_json::from_value::<TgMessage>(value).is_err());
}

#[test]
fn excessive_update_list_is_rejected_during_deserialization() {
    let updates = vec![serde_json::json!({"update_id":1,"message":null}); 101];
    let value = serde_json::json!({"ok":true,"result":updates});
    assert!(serde_json::from_value::<TgResponse<TgUpdates>>(value).is_err());
}

fn message(text: &str, offset: u32) -> TgMessage {
    TgMessage {
        message_id: 1,
        message_thread_id: None,
        from: None,
        chat: TgChat {
            id: 1,
            chat_type: "group".into(),
        },
        text: Some(text.into()),
        entities: Some(
            vec![TgEntity {
                entity_type: "mention".into(),
                offset,
                length: 6,
            }]
            .try_into()
            .unwrap(),
        ),
    }
}
