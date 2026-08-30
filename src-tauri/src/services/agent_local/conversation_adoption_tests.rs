use super::conversation_history::ProviderRole;
use super::session_store;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, NonReplayTarget, ReasoningModeId, RouteId,
};

#[tokio::test]
async fn scheduler_conversation_adoption_persists_the_canonical_turn_once() {
    let session = session_store::create_full("Wakeup", "model", "ollama", true, None)
        .await
        .expect("create session");
    let target = ContinuationTarget::Forbidden(NonReplayTarget {
        route_id: RouteId::Ollama,
        model_id: "model".into(),
        reasoning_mode: ReasoningModeId::Off,
    });

    let admitted =
        crate::services::scheduler::admit_wakeup_turn(&session.id, "Inspecte le projet", target)
            .await
            .expect("admit wakeup");
    let saved = session_store::get(&session.id)
        .await
        .expect("reload session");

    assert_eq!(saved.messages.len(), 1);
    assert_eq!(saved.messages[0].id, admitted.user_message_id);
    assert_eq!(saved.messages[0].turn_id, admitted.turn_id);
    assert_eq!(saved.messages[0].role, "user");
    assert_eq!(admitted.history.messages[0].role, ProviderRole::User);
    assert_eq!(admitted.history.messages[0].content, "Inspecte le projet");

    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[test]
fn every_internal_entry_point_adopts_a_canonical_conversation() {
    const MAX_SOURCE_FILES: usize = 4_096;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pending = vec![root];
    let mut stream_entry_points = Vec::new();
    let mut inspected = 0;
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).expect("read source directory") {
            assert!(
                inspected < MAX_SOURCE_FILES,
                "source scan exceeded its bound"
            );
            inspected += 1;
            let entry = entry.expect("source entry");
            if entry.file_type().expect("source type").is_dir() {
                pending.push(entry.path());
                continue;
            }
            if entry.path().extension().and_then(|value| value.to_str()) != Some("rs") {
                continue;
            }
            let source = std::fs::read_to_string(entry.path()).expect("read Rust source");
            if source.contains("run_stream_task(") && !source.contains("fn run_stream_task(") {
                assert!(source.contains("StreamConversation::canonical"));
                assert!(!source.contains("internal_legacy"));
                assert!(!source.contains("session_store::add_messages"));
                stream_entry_points.push(entry.path());
            }
        }
    }
    assert_eq!(
        stream_entry_points.len(),
        4,
        "new stream entry point needs canonical adoption"
    );
    assert!(
        !include_str!("../../commands/agent_chat_task/conversation.rs").contains("InternalLegacy")
    );
}

#[test]
fn production_session_message_writes_stay_behind_canonical_owners() {
    const MAX_SOURCE_FILES: usize = 4_096;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pending = vec![root];
    let mut inspected = 0;
    let mut writers = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).expect("read source directory") {
            assert!(
                inspected < MAX_SOURCE_FILES,
                "source scan exceeded its bound"
            );
            inspected += 1;
            let entry = entry.expect("source entry");
            if entry.file_type().expect("source type").is_dir() {
                pending.push(entry.path());
                continue;
            }
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if path.extension().and_then(|value| value.to_str()) != Some("rs")
                || name.contains("test")
            {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("read Rust source");
            if source.contains("session.messages =") || source.contains("session.messages.push(") {
                writers.push(name.to_string());
            }
        }
    }
    writers.sort();
    // Admission and the atomic checkpoint transaction own live writes. Migration
    // repairs legacy data, while session_ops owns the explicit retry/clone boundary.
    assert_eq!(
        writers,
        [
            "checkpoint_transaction.rs",
            "conversation_admission.rs",
            "session_migration_legacy_history.rs",
            "session_ops.rs",
        ]
    );
}
