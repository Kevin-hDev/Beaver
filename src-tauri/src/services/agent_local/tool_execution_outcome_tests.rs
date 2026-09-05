use super::{ToolExecutionOutcome, MAX_FOLLOW_UP_BYTES};
use crate::services::agent_local::tool_artifact::{
    ArtifactMetadata, ArtifactPurpose, ArtifactSource, EphemeralArtifact,
};
use crate::services::agent_local::tool_execution_artifacts::MAX_OUTCOME_ARTIFACTS;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolFollowUp;

#[test]
fn trusted_follow_ups_are_appended_after_tool_messages() {
    let mut outcome = ToolExecutionOutcome::default();
    outcome.record(ToolFollowUp::UserMessage("User answer".into()));
    outcome.record(ToolFollowUp::SystemMessage("Backend state".into()));
    let mut messages = vec![ChatMessage::tool("Receipt".into(), None, None)];

    assert!(!outcome.apply_follow_ups(&mut messages).unwrap());
    assert_eq!(messages.len(), 1);
    assert!(messages[0].content.contains("User answer"));
    assert!(messages[0].content.contains("Backend state"));
}

#[test]
fn stop_follow_up_ends_the_batch_without_fabricating_a_message() {
    let mut outcome = ToolExecutionOutcome::default();
    outcome.record(ToolFollowUp::Stop);
    let mut messages = Vec::new();

    assert!(outcome.apply_follow_ups(&mut messages).unwrap());
    assert!(messages.is_empty());
}

#[test]
fn oversized_follow_up_is_bounded_without_failing_a_large_tool_result() {
    let mut outcome = ToolExecutionOutcome::default();
    outcome.record(ToolFollowUp::UserMessage("é".repeat(MAX_FOLLOW_UP_BYTES)));
    let mut messages = vec![ChatMessage::tool(
        "x".repeat(MAX_FOLLOW_UP_BYTES * 2),
        None,
        None,
    )];

    assert!(!outcome.apply_follow_ups(&mut messages).unwrap());
    assert!(messages[0].content.ends_with('é'));
    assert!(messages[0].content.len() <= MAX_FOLLOW_UP_BYTES * 3 + 32);
}

#[test]
fn follow_up_without_tool_is_ignored_without_failing_the_turn() {
    let mut outcome = ToolExecutionOutcome::default();
    outcome.record(ToolFollowUp::UserMessage("answer".into()));

    assert!(!outcome.apply_follow_ups(&mut []).unwrap());
}

#[test]
fn attributed_artifacts_survive_follow_up_application() {
    let mut outcome = ToolExecutionOutcome::default();
    outcome.record(ToolFollowUp::Stop);
    outcome
        .record_artifacts(3, Some("call-3"), vec![artifact()])
        .unwrap();

    assert!(outcome.apply_follow_ups(&mut []).unwrap());
    let artifacts = outcome.artifacts();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].tool_call_index, 3);
    assert_eq!(artifacts[0].tool_call_id.as_deref(), Some("call-3"));
    assert_eq!(artifacts[0].artifact.bytes, [1, 2, 3]);
}

#[test]
fn attributed_artifacts_are_bounded() {
    let mut outcome = ToolExecutionOutcome::default();
    assert!(outcome
        .record_artifacts(0, None, vec![artifact(); MAX_OUTCOME_ARTIFACTS])
        .is_ok());
    assert!(outcome.record_artifacts(1, None, vec![artifact()]).is_err());
}

#[test]
fn merge_accepts_the_limit_and_rejects_one_more_atomically() {
    let mut outcome = ToolExecutionOutcome::default();
    outcome
        .record_artifacts(0, None, vec![artifact(); MAX_OUTCOME_ARTIFACTS - 1])
        .unwrap();
    let mut last = ToolExecutionOutcome::default();
    last.record_artifacts(1, Some("last"), vec![artifact()]).unwrap();

    assert!(outcome.merge(last).is_ok());
    assert_eq!(outcome.artifacts().len(), MAX_OUTCOME_ARTIFACTS);

    let mut overflow = ToolExecutionOutcome::with_compressed(true);
    overflow.record(ToolFollowUp::UserMessage("must not merge".into()));
    overflow.record_artifacts(2, Some("overflow"), vec![artifact()]).unwrap();
    assert!(outcome.merge(overflow).is_err());
    assert_eq!(outcome.artifacts().len(), MAX_OUTCOME_ARTIFACTS);
    assert!(!outcome.compressed);
    let mut messages = vec![ChatMessage::tool("receipt".into(), None, None)];
    assert!(!outcome.apply_follow_ups(&mut messages).unwrap());
    assert_eq!(messages[0].content, "receipt");
}

fn artifact() -> EphemeralArtifact {
    EphemeralArtifact {
        metadata: ArtifactMetadata {
            name: "report.txt".into(),
            mime_type: "text/plain".into(),
            bytes: 3,
            sha256: "a".repeat(64),
            purpose: ArtifactPurpose::Artifact,
            source: ArtifactSource::WorkspaceFile {
                path: std::path::PathBuf::from("/workspace/report.txt"),
                grant: "grant".into(),
            },
        },
        bytes: vec![1, 2, 3],
    }
}
