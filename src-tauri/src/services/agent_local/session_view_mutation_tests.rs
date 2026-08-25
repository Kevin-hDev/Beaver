use crate::models::agent_session_contract::{EditUserMessageInput, SessionMetadataPatch};
use crate::services::reasoning_continuity::envelope::CompletionState;

use super::session_view_test_support::{fixture_session, responses_envelope};

#[tokio::test]
async fn metadata_patch_preserves_the_canonical_envelope_byte_for_byte() {
    let mut session = super::session_store::create_full(
        "Metadata fixture",
        "gpt-5.6-luna",
        "codex-oauth",
        false,
        None,
    )
    .await
    .expect("create session");
    let mut assistant = fixture_session().messages[1].clone();
    assistant.continuation = Some(responses_envelope(CompletionState::Complete));
    session.messages.push(assistant);
    super::session_store::save(&session).await.expect("seed continuation");
    let before = serde_json::to_vec(session.messages[0].continuation.as_ref().unwrap()).unwrap();

    super::session_ops::apply_metadata_patch(
        &session.id,
        SessionMetadataPatch {
            name: Some("Nom modifié".into()),
            model: Some("gpt-5.6-sol".into()),
            provider: Some("codex-oauth".into()),
            reasoning_mode: Some("high".into()),
            fast_mode_enabled: Some(true),
            project_id: Some("00000000-0000-4000-8000-000000000099".into()),
        },
    )
    .await
    .expect("apply bounded metadata");

    let restored = super::session_store::get(&session.id).await.expect("reload session");
    let after = serde_json::to_vec(restored.messages[0].continuation.as_ref().unwrap()).unwrap();
    assert_eq!(after, before);
    assert_eq!(restored.name, "Nom modifié");
    assert_eq!(restored.project_id.as_deref(), Some("00000000-0000-4000-8000-000000000099"));
    super::session_store::delete_one(&session.id).await.expect("cleanup");
}

#[tokio::test]
async fn edit_input_changes_only_the_targeted_user_content() {
    let mut session = super::session_store::create_full(
        "Edit fixture",
        "fixture-model",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create session");
    let mut user = fixture_session().messages[0].clone();
    user.id = "00000000-0000-4000-8000-000000000010".into();
    user.turn_id = "turn-edit".into();
    user.content = "ancien texte".into();
    let original_files = user.files.clone();
    let mut assistant = fixture_session().messages[1].clone();
    assistant.id = "00000000-0000-4000-8000-000000000011".into();
    assistant.turn_id = "turn-edit".into();
    assistant.continuation = Some(responses_envelope(CompletionState::Complete));
    session.messages = vec![user, assistant];
    super::session_store::save(&session).await.expect("seed edit session");

    super::session_ops::edit_user_message(
        &session.id,
        EditUserMessageInput {
            message_id: "00000000-0000-4000-8000-000000000010".into(),
            new_content: "nouveau texte".into(),
        },
    )
    .await
    .expect("edit user");

    let restored = super::session_store::get(&session.id).await.expect("reload edit");
    assert_eq!(restored.messages.len(), 1);
    assert_eq!(restored.messages[0].id, "00000000-0000-4000-8000-000000000010");
    assert_eq!(restored.messages[0].turn_id, "turn-edit");
    assert_eq!(restored.messages[0].content, "nouveau texte");
    assert_eq!(
        serde_json::to_value(&restored.messages[0].files).unwrap(),
        serde_json::to_value(original_files).unwrap()
    );
    assert!(restored.messages[0].tool_calls.is_none());
    assert!(restored.messages[0].continuation.is_none());
    super::session_store::delete_one(&session.id).await.expect("cleanup");
}
