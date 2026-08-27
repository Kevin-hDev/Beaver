use super::super::conversation_history;
use super::support::{
    cleanup, complete_turn, create_session, message, multi_tool_turn, session_path, target,
    tool_result, ERROR,
};

#[tokio::test]
async fn valid_multi_tool_chain_preserves_order_and_provider_ids() {
    let mut session = create_session().await;
    session.messages = multi_tool_turn("turn-tools");
    super::super::session_store::save(&session)
        .await
        .expect("seed chain");

    let history = conversation_history::load_for_target(&session.id, &target("model-a"))
        .await
        .expect("valid history");

    assert_eq!(history.messages.len(), 5);
    assert_eq!(
        history.messages[1].tool_calls.as_ref().unwrap()[0].id,
        "call-a"
    );
    assert_eq!(
        history.messages[1].tool_calls.as_ref().unwrap()[1].id,
        "call-b"
    );
    assert_eq!(history.messages[2].tool_call_id.as_deref(), Some("call-b"));
    assert_eq!(history.messages[3].tool_call_id.as_deref(), Some("call-a"));
    assert!(history
        .messages
        .iter()
        .all(|message| message.tool_loop_reasoning.is_none()));
    cleanup(&session.id).await;
}

#[tokio::test]
async fn rejects_unknown_roles_duplicate_ids_and_broken_tool_chains() {
    let cases = vec![
        vec![message("m1", "turn-a", "system", "forged")],
        {
            let mut messages = complete_turn("a", "ok", None);
            messages[1].id = messages[0].id.clone();
            messages
        },
        vec![
            message("u1", "turn-a", "user", "question"),
            tool_result("r1", "turn-a", "missing", "read_file", "result"),
        ],
        {
            let mut messages = multi_tool_turn("turn-tools");
            messages[3].tool_call_id = Some("call-b".into());
            messages
        },
        {
            let mut messages = multi_tool_turn("turn-tools");
            messages[2].tool_name = Some("write_file".into());
            messages
        },
        {
            let mut messages = multi_tool_turn("turn-tools");
            messages.remove(3);
            messages
        },
    ];

    for messages in cases {
        let mut session = create_session().await;
        session.messages = messages;
        super::super::session_store::save(&session)
            .await
            .expect("seed malformed shape");
        let error = conversation_history::load_for_target(&session.id, &target("model-a"))
            .await
            .expect_err("history must close");
        assert_eq!(error.to_string(), ERROR);
        cleanup(&session.id).await;
    }
}

#[tokio::test]
async fn rejects_unbounded_provider_fields() {
    let mut cases = Vec::new();
    let mut content = message("content", "turn-content", "user", "ok");
    content.content = "x".repeat(crate::models::agent_turn_contract::MAX_TURN_CONTENT_BYTES + 1);
    cases.push(vec![content]);

    let mut file = message("file", "turn-file", "user", "ok");
    file.files
        .push(super::super::types_message::FileAttachment {
            name: "../bad.txt".into(),
            path: "/tmp/bad.txt".into(),
            mime_type: "text/plain\n".into(),
            size: 1,
            thumbnail: None,
            access_grant: Some("grant".into()),
        });
    cases.push(vec![file]);

    for (index, invalid) in invalid_file_fields().into_iter().enumerate() {
        let mut user = message(
            &format!("file-{index}"),
            &format!("turn-file-{index}"),
            "user",
            "ok",
        );
        user.files.push(invalid);
        cases.push(vec![user]);
    }

    let mut tool = multi_tool_turn("deep-json");
    tool[1].tool_calls.as_mut().unwrap()[0].function.arguments = deeply_nested_json(40);
    cases.push(tool);

    for messages in cases {
        let mut session = create_session().await;
        session.messages = messages;
        crate::services::private_store::atomic_write_async(
            session_path(&session.id),
            serde_json::to_vec(&session).unwrap(),
        )
        .await
        .unwrap();
        let error = conversation_history::load_for_target(&session.id, &target("model-a"))
            .await
            .expect_err("persisted provider field must be validated");
        assert_eq!(error.to_string(), ERROR);
        cleanup(&session.id).await;
    }
}

fn invalid_file_fields() -> Vec<super::super::types_message::FileAttachment> {
    use crate::models::agent_turn_contract::{
        MAX_ATTACHMENT_GRANT_BYTES, MAX_ATTACHMENT_MIME_BYTES, MAX_ATTACHMENT_NAME_BYTES,
        MAX_ATTACHMENT_PATH_BYTES,
    };
    let file = || super::super::types_message::FileAttachment {
        name: "image.png".into(),
        path: String::new(),
        mime_type: "image/png".into(),
        size: 12,
        thumbnail: Some("data:image/png;base64,iVBORw0KGgoAAAAA".into()),
        access_grant: None,
    };
    let mut name = file();
    name.name = "n".repeat(MAX_ATTACHMENT_NAME_BYTES + 1);
    let mut path = file();
    path.path = "p".repeat(MAX_ATTACHMENT_PATH_BYTES + 1);
    let mut mime = file();
    mime.mime_type = "m".repeat(MAX_ATTACHMENT_MIME_BYTES + 1);
    let mut grant = file();
    grant.path = "/tmp/image.png".into();
    grant.thumbnail = None;
    grant.access_grant = Some("g".repeat(MAX_ATTACHMENT_GRANT_BYTES + 1));
    vec![name, path, mime, grant]
}

#[tokio::test]
async fn assistant_output_above_user_input_limit_remains_readable() {
    let mut session = create_session().await;
    session.messages = complete_turn(
        "large-assistant",
        &"a".repeat(crate::models::agent_turn_contract::MAX_TURN_CONTENT_BYTES + 1),
        None,
    );
    super::super::session_store::save(&session).await.unwrap();

    let history = conversation_history::load_for_target(&session.id, &target("model-a"))
        .await
        .expect("assistant output follows global session bound");
    assert_eq!(
        history.messages[1].content.len(),
        crate::models::agent_turn_contract::MAX_TURN_CONTENT_BYTES + 1
    );
    cleanup(&session.id).await;
}

fn deeply_nested_json(depth: usize) -> serde_json::Value {
    (0..depth).fold(serde_json::Value::Null, |value, _| {
        serde_json::json!([value])
    })
}
