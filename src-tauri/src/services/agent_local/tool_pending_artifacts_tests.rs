use super::*;
use crate::services::agent_local::tool_artifact::{ArtifactPurpose, PendingArtifact};

const KEY: [u8; 32] = [3; 32];

fn pending(path: &str) -> PendingArtifact {
    PendingArtifact::from_validated(path.to_string(), None, ArtifactPurpose::Artifact)
}

#[test]
fn resolves_all_admitted_files_before_attaching_them() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("one.txt"), b"one").unwrap();
    std::fs::write(root.path().join("two.txt"), b"two").unwrap();
    let mut result = ToolResult::ok("done");
    result.set_pending_artifacts(vec![pending("one.txt"), pending("two.txt")]).unwrap();

    let result = resolve_with_key(result, root.path(), &CancellationToken::new(), &KEY);

    assert!(!result.is_error);
    assert_eq!(result.ephemeral_artifacts().len(), 2);
    assert!(result.pending_artifacts().is_empty());
    assert_eq!(result.ephemeral_artifacts()[0].metadata.name, "one.txt");
}

#[test]
fn cancellation_before_reading_keeps_no_partial_artifact() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("one.txt"), b"one").unwrap();
    let mut result = ToolResult::ok("done");
    result.set_pending_artifacts(vec![pending("one.txt")]).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();

    let result = resolve_with_key(result, root.path(), &cancel, &KEY);

    assert_eq!(result.status, super::super::tool_result_contract::ToolResultStatus::Cancelled);
    assert!(result.ephemeral_artifacts().is_empty());
}

#[test]
fn malformed_or_oversized_candidates_use_translated_extension_codes() {
    let root = tempfile::tempdir().unwrap();
    let mut invalid = ToolResult::ok("done");
    invalid.set_pending_artifacts(vec![pending("../outside.txt")]).unwrap();
    let invalid = resolve_with_key(invalid, root.path(), &CancellationToken::new(), &KEY);
    assert_eq!(
        invalid.error.unwrap().code.as_ref(),
        crate::services::extensions::error_codes::RESULT_INVALID
    );

    let oversized = root.path().join("large.bin");
    std::fs::File::create(&oversized)
        .unwrap()
        .set_len(crate::services::extensions::types::MAX_RESULT_BYTES as u64 + 1)
        .unwrap();
    let mut too_large = ToolResult::ok("done");
    too_large.set_pending_artifacts(vec![pending("large.bin")]).unwrap();
    let too_large = resolve_with_key(too_large, root.path(), &CancellationToken::new(), &KEY);
    assert_eq!(
        too_large.error.unwrap().code.as_ref(),
        crate::services::extensions::error_codes::RESULT_TOO_LARGE
    );
}
