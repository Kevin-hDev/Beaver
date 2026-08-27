use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::models::agent_turn_contract::{NewUserTurnInput, SkillReference, TurnAttachmentInput};

use super::super::{conversation_admission, conversation_history, conversation_input};
use super::support::{cleanup, create_session, message, target, ERROR};

const KEY: [u8; 32] = [73; 32];

#[tokio::test]
async fn next_turn_rebuilds_prior_text_image_and_skill_context_from_rust_authorities() {
    let session = create_session().await;
    let fixture = install_context_fixture("history-rebuild");
    let first = conversation_input::resolve_with_key(fixture.input("first visible"), &KEY)
        .await
        .expect("resolve first turn");
    let first =
        conversation_admission::new_turn_with_key(&session.id, first, target("model-a"), &KEY)
            .await
            .expect("admit first turn");
    let mut durable = super::super::session_store::get(&session.id).await.unwrap();
    durable.messages.push(message(
        "00000000-0000-4000-8000-000000000099",
        &first.turn_id,
        "assistant",
        "answer",
    ));
    super::super::session_store::save(&durable).await.unwrap();

    let second = conversation_input::resolve_with_key(
        NewUserTurnInput {
            content: "second visible".into(),
            files: vec![],
            skills: vec![],
        },
        &KEY,
    )
    .await
    .unwrap();
    let second =
        conversation_admission::new_turn_with_key(&session.id, second, target("model-a"), &KEY)
            .await
            .expect("admit second turn");

    let first_user = second
        .history
        .messages
        .iter()
        .find(|item| item.message_id.as_deref() == Some(first.user_message_id.as_str()))
        .expect("durable first user id");
    assert!(first_user.content.contains("historical text exact"));
    assert_eq!(first_user.images.len(), 1);
    let skill = second
        .history
        .messages
        .iter()
        .find(|item| item.content.contains("trusted historical body"))
        .expect("historical skill instruction");
    assert!(skill.content.contains("trusted historical body"));
    let persisted = super::super::session_store::get(&session.id).await.unwrap();
    assert_eq!(
        persisted.messages[0].skill_ids.as_deref(),
        Some(&[fixture.skill_id.clone()][..])
    );
    assert_eq!(persisted.messages[0].id, first.user_message_id);
    let visible =
        serde_json::to_string(&super::super::session_view::from_session(&persisted).unwrap())
            .unwrap();
    assert!(!visible.contains(&fixture.skill_id));

    fixture.cleanup();
    cleanup(&session.id).await;
}

#[tokio::test]
async fn legacy_skill_names_without_private_ids_stay_readable_without_invented_context() {
    let mut session = create_session().await;
    let mut legacy = message("legacy-skill", "legacy-turn", "user", "visible only");
    legacy.skill_names = Some(vec!["Legacy visible name".into()]);
    session.messages.push(legacy);
    super::super::session_store::save(&session).await.unwrap();

    let history = conversation_history::load_for_target(&session.id, &target("model-a"))
        .await
        .expect("legacy message remains readable");
    assert_eq!(history.messages.len(), 1);
    assert_eq!(history.messages[0].content, "visible only");
    cleanup(&session.id).await;
}

#[tokio::test]
async fn unavailable_historical_file_or_skill_blocks_with_one_generic_error() {
    for remove_skill in [false, true] {
        let session = create_session().await;
        let fixture = install_context_fixture(if remove_skill {
            "missing-skill"
        } else {
            "missing-file"
        });
        let input = conversation_input::resolve_with_key(fixture.input("visible"), &KEY)
            .await
            .unwrap();
        conversation_admission::new_turn_with_key(&session.id, input, target("model-a"), &KEY)
            .await
            .unwrap();
        if remove_skill {
            fs::remove_dir_all(&fixture.skill_root).unwrap();
        } else {
            fs::remove_file(&fixture.text_path).unwrap();
        }

        let error =
            conversation_history::load_for_target_with_key(&session.id, &target("model-a"), &KEY)
                .await
                .expect_err("missing authority must close");
        assert_eq!(error.to_string(), ERROR);
        fixture.cleanup();
        cleanup(&session.id).await;
    }
}

#[tokio::test]
async fn unavailable_prior_context_blocks_admission_before_mutating_the_session() {
    let session = create_session().await;
    let fixture = install_context_fixture("admission-preflight");
    let first = conversation_input::resolve_with_key(fixture.input("first"), &KEY)
        .await
        .unwrap();
    let admitted =
        conversation_admission::new_turn_with_key(&session.id, first, target("model-a"), &KEY)
            .await
            .unwrap();
    let mut durable = super::super::session_store::get(&session.id).await.unwrap();
    durable.messages.push(message(
        "00000000-0000-4000-8000-000000000098",
        &admitted.turn_id,
        "assistant",
        "answer",
    ));
    super::super::session_store::save(&durable).await.unwrap();
    fs::remove_file(&fixture.text_path).unwrap();
    let path = super::support::session_path(&session.id);
    let before = fs::read(&path).unwrap();
    let next = conversation_input::resolve_with_key(
        NewUserTurnInput {
            content: "next".into(),
            files: vec![],
            skills: vec![],
        },
        &KEY,
    )
    .await
    .unwrap();

    let error =
        conversation_admission::new_turn_with_key(&session.id, next, target("model-a"), &KEY)
            .await
            .expect_err("unavailable prior context blocks admission");

    assert_eq!(error.to_string(), ERROR);
    assert_eq!(fs::read(path).unwrap(), before);
    fixture.cleanup();
    cleanup(&session.id).await;
}

#[tokio::test]
async fn unavailable_context_blocks_edit_before_mutating_the_session() {
    let session = create_session().await;
    let fixture = install_context_fixture("edit-preflight");
    let first = conversation_input::resolve_with_key(fixture.input("first"), &KEY)
        .await
        .unwrap();
    let admitted =
        conversation_admission::new_turn_with_key(&session.id, first, target("model-a"), &KEY)
            .await
            .unwrap();
    fs::remove_file(&fixture.text_path).unwrap();
    let path = super::support::session_path(&session.id);
    let before = fs::read(&path).unwrap();

    let error = super::super::conversation_edit::edit_user_message(
        &session.id,
        crate::models::agent_session_contract::EditUserMessageInput {
            message_id: admitted.user_message_id,
            new_content: "edited".into(),
        },
        &target("model-a"),
    )
    .await
    .expect_err("unavailable context blocks edit before its write");

    assert_eq!(error.to_string(), ERROR);
    assert_eq!(fs::read(path).unwrap(), before);
    fixture.cleanup();
    cleanup(&session.id).await;
}

#[tokio::test]
async fn resume_rebuilds_prior_context_before_accepting_a_simple_terminal_user() {
    let session = create_session().await;
    let fixture = install_context_fixture("resume-history");
    let first = conversation_input::resolve_with_key(fixture.input("first"), &KEY)
        .await
        .unwrap();
    let admitted =
        conversation_admission::new_turn_with_key(&session.id, first, target("model-a"), &KEY)
            .await
            .unwrap();
    let mut durable = super::super::session_store::get(&session.id).await.unwrap();
    durable.messages.push(message(
        "00000000-0000-4000-8000-000000000097",
        &admitted.turn_id,
        "assistant",
        "answer",
    ));
    let terminal = message(
        "00000000-0000-4000-8000-000000000096",
        "terminal-simple",
        "user",
        "retry",
    );
    durable.messages.push(terminal.clone());
    super::super::session_store::save(&durable).await.unwrap();

    let resumed = conversation_admission::resume_with_key(
        &session.id,
        crate::models::agent_turn_contract::ResumeTurnInput {
            message_id: terminal.id,
        },
        target("model-a"),
        &KEY,
    )
    .await
    .expect("resume rebuilds all prior Rust context");

    let prior = resumed
        .history
        .messages
        .iter()
        .find(|item| item.message_id.as_deref() == Some(admitted.user_message_id.as_str()))
        .expect("prior durable user");
    assert!(prior.content.contains("historical text exact"));
    assert_eq!(prior.images.len(), 1);
    assert!(resumed
        .history
        .messages
        .iter()
        .any(|item| item.content.contains("trusted historical body")));
    fixture.cleanup();
    cleanup(&session.id).await;
}

#[tokio::test]
async fn edit_race_after_preflight_fails_before_writer_and_preserves_exact_bytes() {
    let session = create_session().await;
    let fixture = install_context_fixture("edit-race");
    let first = conversation_input::resolve_with_key(fixture.input("first"), &KEY)
        .await
        .unwrap();
    let admitted =
        conversation_admission::new_turn_with_key(&session.id, first, target("model-a"), &KEY)
            .await
            .unwrap();
    let path = super::support::session_path(&session.id);
    let before = fs::read(&path).unwrap();
    let remove = fixture.text_path.clone();
    let writer_called = Arc::new(AtomicBool::new(false));
    let observed = writer_called.clone();

    let error = conversation_admission::edit_user_message_after_preflight_with_key_and_writer(
        &session.id,
        crate::models::agent_session_contract::EditUserMessageInput {
            message_id: admitted.user_message_id,
            new_content: "edited".into(),
        },
        &target("model-a"),
        &KEY,
        move || async move {
            fs::remove_file(remove).unwrap();
        },
        move |_| async move {
            observed.store(true, Ordering::SeqCst);
            Ok(())
        },
    )
    .await
    .expect_err("post-preflight race must fail before the writer");

    assert_eq!(error.to_string(), ERROR);
    assert!(!writer_called.load(Ordering::SeqCst));
    assert_eq!(fs::read(path).unwrap(), before);
    fixture.cleanup();
    cleanup(&session.id).await;
}

struct ContextFixture {
    root: std::path::PathBuf,
    text_path: std::path::PathBuf,
    skill_root: std::path::PathBuf,
    skill_id: String,
}

impl ContextFixture {
    fn input(&self, content: &str) -> NewUserTurnInput {
        let raw = self.text_path.to_string_lossy().to_string();
        let registered = crate::services::attachment_access::register_paths(
            std::slice::from_ref(&raw),
            &KEY,
            |_| true,
        )
        .unwrap();
        NewUserTurnInput {
            content: content.into(),
            files: vec![
                TurnAttachmentInput {
                    name: "notes.txt".into(),
                    path: registered[0].path.clone(),
                    mime_type: "text/plain".into(),
                    size: registered[0].size,
                    thumbnail: None,
                    access_grant: Some(registered[0].access_grant.clone()),
                },
                TurnAttachmentInput {
                    name: "image.png".into(),
                    path: String::new(),
                    mime_type: "image/png".into(),
                    size: 12,
                    thumbnail: Some("data:image/png;base64,iVBORw0KGgoAAAAA".into()),
                    access_grant: None,
                },
            ],
            skills: vec![SkillReference {
                id: self.skill_id.clone(),
                name: None,
            }],
        }
    }

    fn cleanup(&self) {
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.skill_root);
    }
}

fn install_context_fixture(label: &str) -> ContextFixture {
    let root = std::env::temp_dir().join(format!("beaver-{label}-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let text_path = root.join("notes.txt");
    fs::write(&text_path, "historical text exact").unwrap();
    let skill_root = crate::services::paths::data_dir()
        .join("skills")
        .join(format!("{label}-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&skill_root).unwrap();
    fs::write(
        skill_root.join("SKILL.md"),
        "---\nname: Historical\ndescription: fixture\n---\ntrusted historical body",
    )
    .unwrap();
    let canonical = skill_root.canonicalize().unwrap();
    let skill_id = super::super::skill_catalog::entries()
        .unwrap()
        .into_iter()
        .find(|entry| entry.bundle_root == canonical)
        .expect("installed skill")
        .info
        .id;
    ContextFixture {
        root,
        text_path,
        skill_root,
        skill_id,
    }
}
