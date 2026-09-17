use super::resource_loader::LoadedResource;
use super::tool_result_media::extension_resource_artifact;

#[test]
fn non_text_resource_keeps_only_extension_provenance() {
    let artifact = extension_resource_artifact(LoadedResource {
        name: "preview.png".into(),
        extension_id: "example.preview".into(),
        qualified_resource_id: "extension:example.preview:image".into(),
        catalog_fingerprint: "a".repeat(64),
        bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
        signature: crate::services::file_signature::FileSignature::Png,
    })
    .expect("artifact conversion")
    .expect("binary resource produces an artifact");

    assert_eq!(artifact.metadata.name, "preview.png");
    assert_eq!(artifact.metadata.mime_type, "image/png");
    assert_eq!(artifact.metadata.bytes, 8);
    assert_eq!(
        artifact.metadata.purpose,
        crate::services::agent_local::tool_artifact::ArtifactPurpose::Artifact
    );
    assert_eq!(
        artifact.metadata.sha256,
        "4c4b6a3be1314ab86138bef4314dde022e600960d8689a2c8f8631802d20dab6"
    );
    let metadata = serde_json::to_string(&artifact.metadata).expect("serializable metadata");
    assert!(!metadata.contains("path"));
    assert!(!metadata.contains("grant"));
    assert!(matches!(
        artifact.metadata.source,
        crate::services::agent_local::tool_artifact::ArtifactSource::ExtensionResource {
            ref resource_id,
            ref catalog_fingerprint,
        } if resource_id == "extension:example.preview:image" && catalog_fingerprint.len() == 64
    ));
}

#[test]
fn text_resource_stays_textual() {
    let artifact = extension_resource_artifact(LoadedResource {
        name: "guide.txt".into(),
        extension_id: "example.preview".into(),
        qualified_resource_id: "extension:example.preview:guide".into(),
        catalog_fingerprint: "b".repeat(64),
        bytes: b"guide".to_vec(),
        signature: crate::services::file_signature::FileSignature::Utf8,
    })
    .expect("text conversion");

    assert!(artifact.is_none());
}
