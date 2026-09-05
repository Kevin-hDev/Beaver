use sha2::{Digest, Sha256};

use super::session_artifact_verification::{resource_status_from_load, verify_workspace};
use super::tool_artifact_record::{
    ToolArtifactPurpose, ToolArtifactRecord, ToolArtifactSource, ToolArtifactStatus,
};

fn workspace(path: String, grant: String, bytes: &[u8]) -> ToolArtifactRecord {
    ToolArtifactRecord {
        name: "artifact.bin".into(),
        mime_type: "application/octet-stream".into(),
        bytes: bytes.len() as u64,
        sha256: hex::encode(Sha256::digest(bytes)),
        purpose: ToolArtifactPurpose::Artifact,
        source: ToolArtifactSource::WorkspaceFile { path, grant },
    }
}

#[tokio::test]
async fn workspace_revalidation_marks_intact_modified_and_inaccessible() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("artifact.bin");
    std::fs::write(&path, b"first").unwrap();
    let key = [7_u8; 32];
    let registered = crate::services::attachment_access::register_paths(
        &[path.to_string_lossy().into_owned()],
        &key,
        |_| true,
    )
    .unwrap();
    let record = workspace(
        registered[0].path.clone(),
        registered[0].access_grant.clone(),
        b"first",
    );
    assert_eq!(
        verify_workspace(&record, Some(&key)).await,
        ToolArtifactStatus::Intact
    );
    std::fs::write(&path, b"second").unwrap();
    assert_eq!(
        verify_workspace(&record, Some(&key)).await,
        ToolArtifactStatus::Modified
    );
    assert_eq!(
        verify_workspace(&record, None).await,
        ToolArtifactStatus::Inaccessible
    );
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        verify_workspace(&record, Some(&key)).await,
        ToolArtifactStatus::Absent
    );
}

#[tokio::test]
async fn history_verification_budget_skips_files_before_reading_them() {
    use super::session_artifact_verification::{verify_workspace_bounded, HistoryVerificationBudget};
    let mut budget = HistoryVerificationBudget::default();
    let record = workspace("/missing".into(), "invalid".into(), b"");
    let mut verified = 0;
    for _ in 0..100 {
        if verify_workspace_bounded(&record, None, &mut budget).await.is_some() {
            verified += 1;
        }
    }
    assert_eq!(verified, 3, "20 MiB per file plus overflow probe; 64 MiB total");
}

#[tokio::test]
async fn intact_workspace_hash_is_case_insensitive() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("artifact.bin");
    std::fs::write(&path, b"first").unwrap();
    let key = [7_u8; 32];
    let registered = crate::services::attachment_access::register_paths(
        &[path.to_string_lossy().into_owned()], &key, |_| true,
    ).unwrap();
    let mut record = workspace(registered[0].path.clone(), registered[0].access_grant.clone(), b"first");
    record.sha256.make_ascii_uppercase();
    assert_eq!(verify_workspace(&record, Some(&key)).await, ToolArtifactStatus::Intact);
}

#[test]
fn resource_revalidation_maps_current_update_disabled_and_removed() {
    let record = ToolArtifactRecord {
        name: "resource.bin".into(),
        mime_type: "application/octet-stream".into(),
        bytes: 5,
        sha256: hex::encode(Sha256::digest(b"first")),
        purpose: ToolArtifactPurpose::Artifact,
        source: ToolArtifactSource::ExtensionResource {
            resource_id: "extension:demo:file".into(),
            catalog_fingerprint: "a".repeat(64),
        },
    };
    let loaded = |fingerprint: String, bytes: Vec<u8>| {
        crate::services::extensions::LoadedResource {
            name: "resource".into(),
            extension_id: "demo".into(),
            qualified_resource_id: "extension:demo:file".into(),
            catalog_fingerprint: fingerprint,
            bytes,
            signature: crate::services::file_signature::FileSignature::Binary,
        }
    };
    assert_eq!(
        resource_status_from_load(&record, Ok(loaded("a".repeat(64), b"first".to_vec()))),
        ToolArtifactStatus::Intact
    );
    assert_eq!(
        resource_status_from_load(&record, Ok(loaded("b".repeat(64), b"first".to_vec()))),
        ToolArtifactStatus::Modified
    );
    assert_eq!(
        resource_status_from_load(
            &record,
            Err(crate::services::extensions::ResourceLoadError::Unavailable)
        ),
        ToolArtifactStatus::Inaccessible
    );
    assert_eq!(
        resource_status_from_load(
            &record,
            Err(crate::services::extensions::ResourceLoadError::NotFound)
        ),
        ToolArtifactStatus::Absent
    );
}
