#[test]
fn extension_resource_metadata_never_serializes_file_access_or_binary_bytes() {
    let metadata = super::tool_artifact::ArtifactMetadata {
        name: "preview.png".into(),
        mime_type: "image/png".into(),
        bytes: 8,
        sha256: "a".repeat(64),
        purpose: super::tool_artifact::ArtifactPurpose::Preview,
        source: super::tool_artifact::ArtifactSource::ExtensionResource {
            resource_id: "extension:example.preview:image".into(),
            catalog_fingerprint: "b".repeat(64),
        },
    };

    let json = serde_json::to_string(&metadata).expect("serializable metadata");
    assert!(!json.contains("path"));
    assert!(!json.contains("grant"));
    assert!(!json.contains("origin"));
    assert!(!json.contains("bytesBase64"));
    assert!(!json.contains("iVBORw0KGgo="));
}
