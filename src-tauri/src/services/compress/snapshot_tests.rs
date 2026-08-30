use std::path::PathBuf;

use super::profile_resolve::resolve_from_document;
use super::profile_store_document::CompressionProfileDocument;
use super::profile_types::CompressionTrigger;
use super::session_capabilities::SessionCompressionCapabilities;
use super::snapshot::CompressionSnapshot;
use crate::services::agent_local::types_session::AgentSession;

pub(super) fn session() -> AgentSession {
    crate::services::agent_local::session_migration::read(
        include_bytes!("../../../test-fixtures/agent-session-v2-compression.json"),
        PathBuf::from("fixture.json"),
    )
    .unwrap()
    .into_session()
}

pub(super) fn snapshot(session: &AgentSession) -> CompressionSnapshot {
    let document = CompressionProfileDocument::default();
    let profile = resolve_from_document(None, &document).unwrap();
    let capabilities = SessionCompressionCapabilities::from_runtime(
        false,
        &["read_file".into(), "web_search".into()],
        true,
        false,
        false,
    )
    .unwrap();
    CompressionSnapshot::capture(
        session,
        profile,
        128_000,
        capabilities,
        CompressionTrigger::Explicit,
    )
    .unwrap()
}

#[test]
fn snapshot_is_an_immutable_copy_of_the_session_messages() {
    let mut session = session();
    let snapshot = snapshot(&session);
    let captured = snapshot.source_messages[0].content.clone();

    session.messages[0].content = "changed later".to_string();

    assert_eq!(snapshot.source_messages[0].content, captured);
    assert_ne!(
        snapshot.source_messages[0].content,
        session.messages[0].content
    );
}

#[test]
fn snapshot_rejects_a_mismatched_unbounded_session_id() {
    let mut session = session();
    session.id = "../outside".to_string();
    let document = CompressionProfileDocument::default();
    let profile = resolve_from_document(None, &document).unwrap();
    let capabilities =
        SessionCompressionCapabilities::from_runtime(false, &[], false, false, false).unwrap();

    assert!(CompressionSnapshot::capture(
        &session,
        profile,
        128_000,
        capabilities,
        CompressionTrigger::Automatic,
    )
    .is_err());
}
