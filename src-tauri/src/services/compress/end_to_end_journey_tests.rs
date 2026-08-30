use std::path::PathBuf;

use super::super::checkpoint_candidate;
use super::super::checkpoint_transaction::{commit_candidate, CompressionError};
use super::super::profile_store::{load_from_paths, trigger_settings};
use crate::services::agent_local::types_message::AgentMessageKind;
use crate::services::agent_local::types_ollama::ChatMessage;

#[tokio::test]
async fn migration_profile_snapshot_and_two_atomic_compressions_survive_restart() {
    let root = tempfile::tempdir().expect("temporary migration root");
    let profile_path = root.path().join("compression-profiles.json");
    let config_path = root.path().join("config.json");
    std::fs::write(
        &config_path,
        br#"{"advanced":{"compression_enabled":false,"compression_threshold":85}}"#,
    )
    .expect("legacy config");
    let mut document = load_from_paths(&profile_path, &config_path).expect("profile migration");
    assert_eq!(document.profiles[0].threshold_percent, 85);
    assert!(!trigger_settings(&document, 32_000).unwrap().available);
    assert!(!document.automatic_enabled);
    document.automatic_enabled = true;
    document.profiles[0].allow_under_64k = true;
    for window in [32_000, 96_000, 200_000] {
        assert!(trigger_settings(&document, window).unwrap().available);
    }

    let legacy_path = root.path().join("session.json");
    let legacy_bytes = include_bytes!("../../../test-fixtures/agent-session-v2-compression.json");
    std::fs::write(&legacy_path, legacy_bytes).expect("legacy session");
    let loaded =
        crate::services::agent_local::session_migration::read(legacy_bytes, legacy_path.clone())
            .expect("session migration");
    assert!(loaded
        .session()
        .messages
        .iter()
        .any(|message| { message.message_kind == Some(AgentMessageKind::CompressionCheckpoint) }));
    crate::services::agent_local::session_migration::commit_current(&loaded)
        .await
        .expect("persist v3");
    let current = std::fs::read(&legacy_path).expect("current session");
    let reloaded = crate::services::agent_local::session_migration::read(
        &current,
        PathBuf::from(&legacy_path),
    )
    .expect("reload v3");
    assert_eq!(
        reloaded.version(),
        crate::services::agent_local::session_migration::LoadedVersion::V3
    );

    let fixture = reloaded.into_session();
    let mut stored = crate::services::agent_local::session_store::create_full(
        "compression e2e",
        &fixture.model,
        &fixture.provider,
        false,
        None,
    )
    .await
    .expect("create session");
    stored.messages = fixture.messages;
    crate::services::agent_local::session_store::save(&stored)
        .await
        .expect("save fixture");

    let frozen = super::support::snapshot(&stored, &document, 96_000);
    let frozen_threshold = frozen.profile.profile.threshold_percent;
    document.profiles[0].threshold_percent = 40;
    assert_eq!(frozen.profile.profile.threshold_percent, frozen_threshold);
    let first = checkpoint_candidate::build(&frozen, Some(&super::support::summary()), &[])
        .await
        .expect("first candidate");
    let original = serde_json::to_vec(&stored).expect("original session");
    let mut runtime = vec![ChatMessage::user("live runtime".into())];
    crate::services::agent_local::session_store::fail_next_prepared_save();
    assert!(matches!(
        commit_candidate(&stored.id, &mut runtime, first).await,
        Err(CompressionError::SaveFailed)
    ));
    let unchanged = crate::services::agent_local::session_store::get(&stored.id)
        .await
        .expect("unchanged session");
    assert_eq!(serde_json::to_vec(&unchanged).unwrap(), original);

    let first = checkpoint_candidate::build(&frozen, Some(&super::support::summary()), &[])
        .await
        .expect("rebuilt first candidate");
    commit_candidate(&stored.id, &mut runtime, first)
        .await
        .expect("first commit");
    let after_first = crate::services::agent_local::session_store::get(&stored.id)
        .await
        .expect("first reload");
    let second_snapshot = super::support::snapshot(&after_first, &document, 200_000);
    let second =
        checkpoint_candidate::build(&second_snapshot, Some(&super::support::summary()), &[])
            .await
            .expect("second candidate");
    commit_candidate(&stored.id, &mut runtime, second)
        .await
        .expect("second commit");
    let after_restart = crate::services::agent_local::session_store::get(&stored.id)
        .await
        .expect("second reload");
    assert_eq!(
        after_restart.compression_count,
        stored.compression_count + 2
    );
    assert_eq!(
        after_restart
            .messages
            .iter()
            .filter_map(|message| message.message_kind)
            .collect::<Vec<_>>(),
        vec![
            AgentMessageKind::CompressionCheckpoint,
            AgentMessageKind::CompressionBoundary,
        ]
    );
    crate::services::agent_local::session_store::delete_one(&stored.id)
        .await
        .expect("cleanup");
}
