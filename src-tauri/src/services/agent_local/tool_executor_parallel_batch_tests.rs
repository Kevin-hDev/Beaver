use super::*;
use crate::services::agent_local::tool_artifact::{ArtifactPurpose, PendingArtifact};
use crate::services::agent_local::tool_execution_outcome::ToolExecutionOutcome;

const KEY: [u8; 32] = [9; 32];
const TWENTY_MIB: u64 = 20 * 1024 * 1024;

#[tokio::test]
async fn flushes_two_read_chunks_before_one_shared_artifact_budget() {
    let root = tempfile::tempdir().expect("root");
    let calls: Vec<_> = (0..(MAX_PARALLEL + 1))
        .map(|index| ("read_file".to_owned(), serde_json::json!({ "path": format!("{index}.bin") })))
        .collect();
    let entries: Vec<_> = calls
        .iter()
        .enumerate()
        .map(|(global_idx, (name, effective_args))| BatchEntry {
            global_idx,
            name,
            effective_args,
        })
        .collect();
    let mut eager = std::collections::HashMap::new();
    for index in 0..calls.len() {
        let path = format!("{index}.bin");
        std::fs::File::create(root.path().join(&path))
            .expect("file")
            .set_len(TWENTY_MIB)
            .expect("size");
        let mut result = ToolResult::ok(format!("result {index}"));
        result
            .set_pending_artifacts(vec![PendingArtifact::from_validated(
                path,
                None,
                ArtifactPurpose::Artifact,
            )])
            .expect("pending artifact");
        eager.insert(index, result);
    }
    let cancel = CancellationToken::new();
    let mut indexed_results = vec![None; calls.len()];
    let mut write_guard = WriteGuard::new();
    flush_read_batch(
        &entries,
        &mut indexed_results,
        root.path(),
        &cancel,
        &mut write_guard,
        &mut Some(&mut eager),
        "test-session",
        "test-request",
        true,
    )
    .await;

    assert!(indexed_results
        .iter()
        .flatten()
        .all(|(_, result)| !result.pending_artifacts().is_empty()));
    super::super::tool_executor_parallel_finalize::resolve_with_test_key(
        &mut indexed_results,
        root.path(),
        &cancel,
        &KEY,
    );
    assert!(indexed_results
        .iter()
        .take(3)
        .all(|slot| !slot.as_ref().expect("result").1.is_error));
    assert!(indexed_results
        .iter()
        .skip(3)
        .all(|slot| slot.as_ref().expect("result").1.is_error));

    let mut outcome = ToolExecutionOutcome::default();
    for (index, slot) in indexed_results.iter_mut().enumerate().take(3) {
        let (_, result) = slot.as_mut().expect("admitted result");
        outcome
            .record_artifacts(index, Some(&format!("call-{index}")), result.take_ephemeral_artifacts())
            .expect("bounded artifacts");
    }
    let artifacts = outcome.artifacts();
    assert_eq!(artifacts.len(), 3);
    for (index, artifact) in artifacts.iter().enumerate() {
        assert_eq!(artifact.tool_call_index, index);
        assert_eq!(artifact.tool_call_id.as_deref(), Some(format!("call-{index}").as_str()));
    }
}
