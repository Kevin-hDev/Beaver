use super::*;
use crate::services::agent_local::tool_artifact::{ArtifactPurpose, PendingArtifact};
use crate::services::agent_local::tool_execution_outcome::ToolExecutionOutcome;

const KEY: [u8; 32] = [5; 32];

#[test]
fn read_write_read_results_share_resolution_and_keep_attribution_order() {
    let root = tempfile::tempdir().expect("root");
    let names = ["read_file", "extension.create", "read_file"];
    let ids = ["call-read-1", "call-write", "call-read-2"];
    let mut indexed_results = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let path = format!("artifact-{index}.bin");
        std::fs::write(root.path().join(&path), [index as u8]).expect("artifact");
        let mut result = crate::services::agent_local::types_tools::ToolResult::ok("done");
        result
            .set_pending_artifacts(vec![PendingArtifact::from_validated(
                path,
                None,
                ArtifactPurpose::Artifact,
            )])
            .expect("pending artifact");
        indexed_results.push(Some((*name, result)));
    }

    resolve_with_test_key(
        &mut indexed_results,
        root.path(),
        &tokio_util::sync::CancellationToken::new(),
        &KEY,
    );

    let mut outcome = ToolExecutionOutcome::default();
    for (index, slot) in indexed_results.iter_mut().enumerate() {
        let (name, result) = slot.as_mut().expect("resolved result");
        assert_eq!(*name, names[index]);
        assert!(!result.is_error);
        outcome
            .record_artifacts(index, Some(ids[index]), result.take_ephemeral_artifacts())
            .expect("attributed artifact");
    }
    let artifacts = outcome.artifacts();
    assert_eq!(artifacts.len(), 3);
    for (index, artifact) in artifacts.iter().enumerate() {
        assert_eq!(artifact.tool_call_index, index);
        assert_eq!(artifact.tool_call_id.as_deref(), Some(ids[index]));
        assert_eq!(artifact.artifact.bytes, [index as u8]);
    }
}
