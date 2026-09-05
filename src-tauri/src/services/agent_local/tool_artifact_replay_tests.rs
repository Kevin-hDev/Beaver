use sha2::{Digest, Sha256};

use super::super::tool_artifact_record::{
    ToolArtifactPurpose, ToolArtifactRecord, ToolArtifactSource, ToolArtifactStatus,
};
use super::{replay, replay_extension_from_load, ArtifactReplay};

fn record(source: ToolArtifactSource, bytes: &[u8]) -> ToolArtifactRecord {
    ToolArtifactRecord {
        name: "saved-output.bin".into(),
        mime_type: "application/octet-stream".into(),
        bytes: bytes.len() as u64,
        sha256: hex::encode(Sha256::digest(bytes)),
        purpose: ToolArtifactPurpose::Preview,
        source,
    }
}

fn assert_note(replay: ArtifactReplay, status: ToolArtifactStatus) {
    assert_eq!(replay.status, status);
    assert!(replay.artifact.is_none());
    assert!(replay.note.is_some());
}

#[tokio::test]
async fn workspace_replay_revalidates_intact_absent_modified_and_inaccessible() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("saved-output.bin");
    std::fs::write(&path, b"first").unwrap();
    let key = [7_u8; 32];
    let registered = crate::services::attachment_access::register_paths(
        &[path.to_string_lossy().into_owned()],
        &key,
        |_| true,
    )
    .unwrap();
    let record = record(
        ToolArtifactSource::WorkspaceFile {
            path: registered[0].path.clone(),
            grant: registered[0].access_grant.clone(),
        },
        b"first",
    );

    let intact = replay(&record, Some(&key)).await;
    assert_eq!(intact.status, ToolArtifactStatus::Intact);
    assert_eq!(intact.artifact.unwrap().bytes.as_ref(), b"first");

    std::fs::write(&path, b"second").unwrap();
    assert_note(
        replay(&record, Some(&key)).await,
        ToolArtifactStatus::Modified,
    );
    assert_note(
        replay(&record, None).await,
        ToolArtifactStatus::Inaccessible,
    );

    std::fs::remove_file(&path).unwrap();
    assert_note(
        replay(&record, Some(&key)).await,
        ToolArtifactStatus::Absent,
    );
}

#[test]
fn extension_replay_maps_intact_absent_modified_and_inaccessible_without_execution() {
    let record = record(
        ToolArtifactSource::ExtensionResource {
            resource_id: "extension:demo:output".into(),
            catalog_fingerprint: "a".repeat(64),
        },
        b"first",
    );
    let loaded =
        |fingerprint: String, bytes: Vec<u8>| crate::services::extensions::LoadedResource {
            name: "output".into(),
            extension_id: "demo".into(),
            qualified_resource_id: "extension:demo:output".into(),
            catalog_fingerprint: fingerprint,
            bytes,
            signature: crate::services::file_signature::FileSignature::Binary,
        };

    let intact = replay_extension_from_load(&record, Ok(loaded("a".repeat(64), b"first".to_vec())));
    assert_eq!(intact.status, ToolArtifactStatus::Intact);
    assert_eq!(intact.artifact.unwrap().bytes.as_ref(), b"first");
    assert_note(
        replay_extension_from_load(&record, Ok(loaded("b".repeat(64), b"first".to_vec()))),
        ToolArtifactStatus::Modified,
    );
    assert_note(
        replay_extension_from_load(
            &record,
            Err(crate::services::extensions::ResourceLoadError::NotFound),
        ),
        ToolArtifactStatus::Absent,
    );
    assert_note(
        replay_extension_from_load(
            &record,
            Err(crate::services::extensions::ResourceLoadError::Unavailable),
        ),
        ToolArtifactStatus::Inaccessible,
    );
}

#[test]
fn replay_drops_invalid_persisted_metadata_before_any_source_is_opened() {
    let mut invalid = record(
        ToolArtifactSource::WorkspaceFile {
            path: "ignored".into(),
            grant: "ignored".into(),
        },
        b"first",
    );
    invalid.name = "\n".into();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    assert_note(
        runtime.block_on(replay(&invalid, None)),
        ToolArtifactStatus::Inaccessible,
    );
}
