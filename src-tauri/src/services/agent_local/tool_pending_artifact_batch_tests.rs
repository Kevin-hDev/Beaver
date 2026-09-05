use super::*;
use crate::services::agent_local::tool_artifact::{
    ArtifactPurpose, ArtifactSource, PendingArtifact, PendingExtensionResource,
};

const KEY: [u8; 32] = [7; 32];
const TWENTY_MIB: u64 = 20 * 1024 * 1024;

#[test]
fn budget_admits_three_twenty_mib_results_and_refuses_only_the_fourth() {
    let root = tempfile::tempdir().expect("root");
    let mut results = Vec::new();
    for index in 0..4 {
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
            .expect("pending");
        results.push(Some(result));
    }

    let mut budget = BatchArtifactBudget::new();
    let results = resolve_with_budget(
        results,
        root.path(),
        &CancellationToken::new(),
        &mut budget,
        Some(&KEY),
    );

    for result in results.iter().take(3).flatten() {
        assert!(!result.is_error);
        assert_eq!(result.ephemeral_artifacts().len(), 1);
    }
    let fourth = results[3].as_ref().expect("fourth result");
    assert_eq!(
        fourth.error.as_ref().map(|error| error.code.as_ref()),
        Some(crate::services::extensions::error_codes::RESULT_TOO_LARGE)
    );
}

#[test]
fn one_budget_bounds_more_than_one_parallel_chunk() {
    let root = tempfile::tempdir().expect("root");
    let mut results = Vec::new();
    for index in 0..(super::super::tool_executor_parallel_batch::MAX_PARALLEL + 1) {
        let path = format!("chunk-{index}.bin");
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
            .expect("pending");
        results.push(Some(result));
    }

    let mut budget = BatchArtifactBudget::new();
    let results = resolve_with_budget(
        results,
        root.path(),
        &CancellationToken::new(),
        &mut budget,
        Some(&KEY),
    );

    assert!(results
        .iter()
        .take(3)
        .all(|result| !result.as_ref().unwrap().is_error));
    assert!(results
        .iter()
        .skip(3)
        .all(|result| result.as_ref().unwrap().is_error));
}

#[test]
fn workspace_and_extension_resources_share_one_batch_budget_in_original_order() {
    let root = tempfile::tempdir().expect("root");
    let mut results = Vec::new();
    for index in 0..4 {
        let path = format!("{index}.bin");
        std::fs::File::create(root.path().join(&path))
            .expect("file")
            .set_len(TWENTY_MIB)
            .expect("size");
        let mut result = ToolResult::ok(format!("result {index}"));
        if index % 2 == 0 {
            result
                .set_pending_artifacts(vec![PendingArtifact::from_validated(
                    path,
                    None,
                    ArtifactPurpose::Artifact,
                )])
                .expect("pending workspace artifact");
        } else {
            result
                .set_pending_extension_resource(PendingExtensionResource {
                    session_id: "test-session".into(),
                    name: format!("resource-{index}.bin"),
                    extension_id: "example.preview".into(),
                    qualified_resource_id: format!("extension:example.preview:{index}"),
                    catalog_fingerprint: "b".repeat(64),
                    root: root.path().to_path_buf(),
                    relative_path: path,
                })
                .expect("pending extension resource");
        }
        results.push(Some(result));
    }

    let mut budget = BatchArtifactBudget::new();
    let results = resolve_with_budget(
        results,
        root.path(),
        &CancellationToken::new(),
        &mut budget,
        Some(&KEY),
    );

    for result in results.iter().take(3).flatten() {
        assert!(!result.is_error);
        assert_eq!(result.ephemeral_artifacts().len(), 1);
    }
    assert!(matches!(
        results[1].as_ref().unwrap().ephemeral_artifacts()[0].metadata.source,
        ArtifactSource::ExtensionResource { ref resource_id, ref catalog_fingerprint }
            if resource_id == "extension:example.preview:1" && catalog_fingerprint == &"b".repeat(64)
    ));
    assert_eq!(
        results[3]
            .as_ref()
            .and_then(|result| result.error.as_ref())
            .map(|error| error.code.as_ref()),
        Some(crate::services::extensions::error_codes::RESULT_TOO_LARGE)
    );
}

#[test]
fn utf8_resource_remains_textual_and_never_serializes_its_internal_path() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("guide.txt"), "Bonjour").expect("text resource");
    let mut result = ToolResult::ok("pending");
    result
        .set_pending_extension_resource(PendingExtensionResource {
            session_id: "test-session".into(),
            name: "guide.txt".into(),
            extension_id: "example.guide".into(),
            qualified_resource_id: "extension:example.guide:guide".into(),
            catalog_fingerprint: "c".repeat(64),
            root: root.path().to_path_buf(),
            relative_path: "guide.txt".into(),
        })
        .expect("pending extension resource");

    let mut budget = BatchArtifactBudget::new();
    let result = resolve_with_budget(
        vec![Some(result)],
        root.path(),
        &CancellationToken::new(),
        &mut budget,
        None,
    )
    .pop()
    .flatten()
    .expect("resolved result");

    assert!(!result.is_error);
    assert!(result.content.contains("Bonjour"));
    assert!(result.ephemeral_artifacts().is_empty());
    let serialized = serde_json::to_string(&result).expect("serialized tool result");
    assert!(!serialized.contains(root.path().to_string_lossy().as_ref()));
    assert!(!serialized.contains("guide.txt"));
}

#[test]
fn missing_workspace_key_does_not_reject_a_neighboring_extension_resource() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("workspace.txt"), b"workspace").expect("workspace file");
    std::fs::write(root.path().join("resource.bin"), [0_u8, 1]).expect("resource file");
    let mut workspace = ToolResult::ok("workspace");
    workspace
        .set_pending_artifacts(vec![PendingArtifact::from_validated(
            "workspace.txt".into(),
            None,
            ArtifactPurpose::Artifact,
        )])
        .expect("pending workspace");
    let mut resource = ToolResult::ok("resource");
    resource
        .set_pending_extension_resource(PendingExtensionResource {
            session_id: "test-session".into(),
            name: "resource.bin".into(),
            extension_id: "example.resource".into(),
            qualified_resource_id: "extension:example.resource:file".into(),
            catalog_fingerprint: "d".repeat(64),
            root: root.path().to_path_buf(),
            relative_path: "resource.bin".into(),
        })
        .expect("pending resource");

    let mut budget = BatchArtifactBudget::new();
    let results = resolve_with_unavailable_workspace_key(
        vec![Some(workspace), Some(resource)],
        root.path(),
        &CancellationToken::new(),
        &mut budget,
    );

    assert!(results[0].as_ref().expect("workspace result").is_error);
    let resource = results[1].as_ref().expect("resource result");
    assert!(!resource.is_error);
    assert_eq!(resource.ephemeral_artifacts().len(), 1);
}

#[tokio::test]
async fn cancellation_before_the_batch_returns_no_partial_artifact() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("report.txt"), b"report").expect("workspace file");
    let mut result = ToolResult::ok("pending");
    result
        .set_pending_artifacts(vec![PendingArtifact::from_validated(
            "report.txt".into(),
            None,
            ArtifactPurpose::Artifact,
        )])
        .expect("pending workspace artifact");
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut results = vec![Some(result)];

    resolve_batch(&mut results, root.path(), &cancellation).await;

    let result = results.pop().flatten().expect("cancelled result");
    assert_eq!(
        result.status,
        crate::services::agent_local::tool_result_contract::ToolResultStatus::Cancelled
    );
    assert!(result.ephemeral_artifacts().is_empty());
}

#[test]
fn repeated_admission_never_exceeds_the_generated_batch_limit() {
    for _ in 0..128 {
        let mut budget = BatchArtifactBudget::new();
        assert!(budget.admit(TWENTY_MIB).is_ok());
        assert!(budget.admit(TWENTY_MIB).is_ok());
        assert!(budget.admit(TWENTY_MIB).is_ok());
        assert!(budget.admit(TWENTY_MIB).is_err());
        assert!(budget.admit(TWENTY_MIB).is_err());
    }
}

#[test]
fn cancellation_during_coordinator_read_keeps_plain_neighbors_and_no_partial_artifact() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("large.bin"), vec![3_u8; 128 * 1024]).expect("artifact file");
    let mut pending = ToolResult::ok("pending");
    pending
        .set_pending_artifacts(vec![PendingArtifact::from_validated(
            "large.bin".into(),
            None,
            ArtifactPurpose::Artifact,
        )])
        .expect("pending artifact");
    let cancel = CancellationToken::new();
    let prepared =
        super::super::tool_pending_artifact_inspect::inspect_result(pending, root.path(), &cancel)
            .expect("inspected result");

    let result = super::super::tool_pending_artifact_read::read_result_cancelling_after_chunk(
        prepared,
        &cancel,
        Some(&KEY),
    );

    assert_eq!(
        result.status,
        crate::services::agent_local::tool_result_contract::ToolResultStatus::Cancelled
    );
    assert!(result.ephemeral_artifacts().is_empty());

    let mut neighbors = vec![Some(ToolResult::ok("plain")), Some(result)];
    cancel_resolving(&mut neighbors, &[false, true]);
    assert_eq!(neighbors[0].as_ref().expect("plain").content, "plain");
    assert_eq!(
        neighbors[1].as_ref().expect("cancelled").status,
        crate::services::agent_local::tool_result_contract::ToolResultStatus::Cancelled
    );
}
