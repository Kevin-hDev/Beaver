use std::sync::Arc;

use tokio::sync::Barrier;
use uuid::Uuid;

use super::super::conversation_admission;
use super::super::conversation_attachments::ResolvedImage;
use super::super::conversation_input::ResolvedTurnInput;
use super::super::conversation_skills::ResolvedSkill;
use super::support::{cleanup, complete_turn, create_session, resolved, session_path, target, ERROR};

#[tokio::test]
async fn persists_user_before_return_and_reserves_three_uuid_v4_ids() {
    let session = create_session().await;
    let observed_id = session.id.clone();
    let admitted = conversation_admission::new_turn_with_after_persist(
        &session.id,
        resolved("question durable"),
        target("model-a"),
        move || async move {
            let persisted = super::super::session_store::get(&observed_id)
                .await
                .expect("user durable before return");
            assert_eq!(persisted.messages.len(), 1);
            assert_eq!(persisted.messages[0].content, "question durable");
        },
    )
    .await
    .expect("admit turn");

    for id in [
        &admitted.turn_id,
        &admitted.user_message_id,
        &admitted.assistant_message_id,
    ] {
        assert_eq!(Uuid::parse_str(id).expect("uuid").get_version_num(), 4);
    }
    assert_ne!(admitted.turn_id, admitted.user_message_id);
    assert_ne!(admitted.turn_id, admitted.assistant_message_id);
    assert_ne!(admitted.user_message_id, admitted.assistant_message_id);

    let persisted = super::super::session_store::get(&session.id)
        .await
        .expect("reload");
    assert_eq!(persisted.messages[0].id, admitted.user_message_id);
    assert_eq!(persisted.messages[0].turn_id, admitted.turn_id);
    assert_eq!(persisted.messages[0].files.len(), 1);
    let source = persisted.messages[0].replay_source.as_ref().expect("durable turn provenance");
    assert_eq!(source.model_id, "model-a");
    assert_eq!(source.route_id, crate::services::reasoning_continuity::contract::RouteId::Ollama);
    assert_eq!(
        persisted.messages[0].skill_names.as_deref(),
        Some(&["Skill local".to_string()][..])
    );
    assert!(persisted
        .messages
        .iter()
        .all(|message| message.id != admitted.assistant_message_id));
    assert_eq!(
        admitted.history.messages.last().unwrap().message_id,
        Some(admitted.user_message_id)
    );
    let visible = serde_json::to_value(
        super::super::session_view::from_session(&persisted).unwrap(),
    ).unwrap();
    assert!(visible["messages"][0].get("replay_source").is_none());
    cleanup(&session.id).await;
}

#[tokio::test]
async fn failed_write_is_generic_and_preserves_previous_bytes() {
    let mut session = create_session().await;
    session.messages = complete_turn("seed", "visible", None);
    super::super::session_store::save(&session)
        .await
        .expect("seed");
    let path = session_path(&session.id);
    let before = std::fs::read(&path).expect("read before");

    let error = conversation_admission::new_turn_with_writer(
        &session.id,
        resolved("never durable"),
        target("model-a"),
        |_| async { Err("technical path /private/session.json".to_string()) },
    )
    .await
    .expect_err("write must fail");

    assert_eq!(error.to_string(), ERROR);
    assert_eq!(std::fs::read(path).expect("read after"), before);
    cleanup(&session.id).await;
}

#[tokio::test]
async fn concurrent_admissions_serialize_without_loss() {
    let session = create_session().await;
    let barrier = Arc::new(Barrier::new(2));
    let first_id = session.id.clone();
    let second_id = session.id.clone();
    let first_barrier = Arc::clone(&barrier);
    let first = tokio::spawn(async move {
        conversation_admission::new_turn_with_after_load(
            &first_id,
            resolved("first"),
            target("model-a"),
            move || async move {
                first_barrier.wait().await;
            },
        )
        .await
    });
    barrier.wait().await;
    let second = tokio::spawn(async move {
        conversation_admission::new_turn(&second_id, resolved("second"), target("model-a")).await
    });
    let first = first.await.expect("join first").expect("first admission");
    assert!(second.await.expect("join second").is_err());

    let persisted = super::super::session_store::get(&session.id)
        .await
        .expect("reload");
    assert_eq!(persisted.messages.len(), 1);
    assert_eq!(persisted.messages[0].id, first.user_message_id);
    cleanup(&session.id).await;
}

#[tokio::test]
async fn limit_refuses_admission_without_draining_a_turn_fragment() {
    let mut session = create_session().await;
    session.messages = (0..(super::super::session_limits::MAX_MESSAGES_PER_SESSION / 2))
        .flat_map(|index| complete_turn(&format!("limit-{index}"), "answer", None))
        .collect();
    super::super::session_store::save(&session)
        .await
        .expect("seed limit");
    let before = std::fs::read(session_path(&session.id)).expect("before");

    let error = conversation_admission::new_turn(
        &session.id,
        resolved("overflow"),
        target("model-a"),
    )
    .await
    .expect_err("must not drain");
    assert_eq!(error.to_string(), ERROR);
    assert_eq!(std::fs::read(session_path(&session.id)).unwrap(), before);
    cleanup(&session.id).await;
}

#[tokio::test]
async fn current_provider_snapshot_keeps_resolved_content_images_and_ordered_skills() {
    let session = create_session().await;
    let redactable = ["sk", "-proj-", "abcdefghijklmnopqrstuvwxyz"].concat();
    let input = ResolvedTurnInput {
        user_content: redactable.clone(),
        provider_content: format!("{redactable}\n\n--- File: exact.txt ---\nresolved exact"),
        files: Vec::new(),
        images: vec![ResolvedImage {
            mime_type: "image/png".into(),
            base64: "exact-image-base64".into(),
        }],
        skills: vec![
            ResolvedSkill {
                id: "local:first".into(),
                name: "First".into(),
                content: "first exact body".into(),
            },
            ResolvedSkill {
                id: "local:second".into(),
                name: "Second".into(),
                content: "second exact body".into(),
            },
        ],
    };

    let admitted = conversation_admission::new_turn(&session.id, input, target("model-a"))
        .await
        .expect("admit rich input");
    let messages = &admitted.history.messages;

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].skill_id.as_deref(), Some("local:first"));
    assert!(messages[0].content.ends_with("first exact body"));
    assert_eq!(messages[1].skill_id.as_deref(), Some("local:second"));
    assert!(messages[1].content.ends_with("second exact body"));
    assert!(messages[0].message_id.is_none() && messages[1].message_id.is_none());
    assert_eq!(
        messages[2].message_id.as_deref(),
        Some(admitted.user_message_id.as_str())
    );
    assert_eq!(messages[2].turn_id, admitted.turn_id);
    assert!(messages[2].content.ends_with("resolved exact"));
    assert_eq!(messages[2].images[0].base64, "exact-image-base64");

    let persisted = super::super::session_store::get(&session.id)
        .await
        .expect("reload durable only");
    assert_eq!(persisted.messages.len(), 1);
    assert_ne!(persisted.messages[0].content, redactable);
    let raw = std::fs::read_to_string(session_path(&session.id)).unwrap();
    assert!(!raw.contains("resolved exact"));
    assert!(!raw.contains("first exact body"));
    assert!(!raw.contains("exact-image-base64"));
    let error = super::super::conversation_history::load_for_target(
        &session.id,
        &target("model-a"),
    )
    .await
    .expect_err("unavailable durable skill ids must fail closed");
    assert_eq!(error.to_string(), ERROR);
    cleanup(&session.id).await;
}
