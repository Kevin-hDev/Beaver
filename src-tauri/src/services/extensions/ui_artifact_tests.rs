use super::types::{ExtensionUiArtifact, ExtensionUiArtifactOutput};
use sha2::{Digest, Sha256};

pub(super) fn fixture(root: &std::path::Path) -> ExtensionUiArtifact {
    let bytes = b"export const ready = true;";
    std::fs::write(root.join("entry.js"), bytes).unwrap();
    let output = ExtensionUiArtifactOutput {
        name: "entry.js".to_string(),
        kind: "javascript".to_string(),
        bytes: bytes.len(),
        sha256: hex::encode(Sha256::digest(bytes)),
    };
    let mut artifact = ExtensionUiArtifact {
        version: 1,
        builder_version: "0.28.1".to_string(),
        node_version: "v20.0.0".to_string(),
        entry: "entry.js".to_string(),
        total_bytes: bytes.len(),
        outputs: vec![output],
        inputs: vec!["entry.ts".to_string()],
        manifest_sha256: "0".repeat(64),
    };
    artifact.manifest_sha256 = super::ui_artifact::manifest_hash(&artifact).unwrap();
    std::fs::write(
        root.join("manifest.json"),
        super::ui_artifact::manifest_bytes(&artifact).unwrap(),
    )
    .unwrap();
    artifact
}

#[test]
fn validates_a_complete_bounded_artifact() {
    let root = tempfile::tempdir().unwrap();
    let artifact = fixture(root.path());

    super::ui_artifact::verify_at(root.path(), &artifact).unwrap();
}

#[test]
fn refuses_output_and_manifest_tampering() {
    let root = tempfile::tempdir().unwrap();
    let artifact = fixture(root.path());
    std::fs::write(root.path().join("entry.js"), b"changed").unwrap();
    assert!(super::ui_artifact::verify_at(root.path(), &artifact).is_err());

    let root = tempfile::tempdir().unwrap();
    let artifact = fixture(root.path());
    std::fs::write(root.path().join("manifest.json"), b"{}").unwrap();
    assert!(super::ui_artifact::verify_at(root.path(), &artifact).is_err());
}

#[test]
fn refuses_unsorted_or_mismatched_manifests() {
    let root = tempfile::tempdir().unwrap();
    let mut artifact = fixture(root.path());
    artifact.inputs = vec!["z.ts".to_string(), "a.ts".to_string()];
    assert!(super::ui_artifact::validate(&artifact).is_err());

    let mut artifact = fixture(root.path());
    artifact.total_bytes += 1;
    assert!(super::ui_artifact::validate(&artifact).is_err());

    let mut artifact = fixture(root.path());
    artifact.outputs[0].name = "style.css".to_string();
    artifact.outputs[0].kind = "css".to_string();
    artifact.entry = "style.css".to_string();
    artifact.manifest_sha256 = super::ui_artifact::manifest_hash(&artifact).unwrap();
    assert!(super::ui_artifact::validate(&artifact).is_err());
}

#[test]
fn cleanup_preserves_an_active_staging_artifact() {
    let staging = super::ui_artifact_store::prepare().unwrap();
    let sentinel = staging.output().join("active.js");
    std::fs::write(&sentinel, b"active").unwrap();

    super::ui_artifact_store::unreferenced(&[]).unwrap();

    assert!(sentinel.is_file());
}

#[cfg(unix)]
#[test]
fn artifact_path_refuses_a_symlinked_extension_directory() {
    use std::os::unix::fs::symlink;

    let id = format!("test.ui.{}", uuid::Uuid::new_v4().simple());
    let root = super::ui_artifact_store::root();
    crate::services::private_store::ensure_private_dir(&root).unwrap();
    let outside = tempfile::tempdir().unwrap();
    let link = root.join(&id);
    symlink(outside.path(), &link).unwrap();

    assert!(super::ui_artifact_store::artifact_path(&id, &"a".repeat(64)).is_err());
    std::fs::remove_file(link).unwrap();
}
