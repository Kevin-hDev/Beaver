use super::tool_result_files::{read_workspace_file, FileResultError};
use crate::services::agent_local::tool_artifact::ArtifactPurpose;

const KEY: [u8; 32] = [9; 32];

#[test]
fn workspace_file_keeps_verified_bytes_metadata_and_grant() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("résultat.txt"), "bonjour").unwrap();
    let artifact = read_workspace_file(
        root.path(),
        "résultat.txt",
        None,
        ArtifactPurpose::Artifact,
        &KEY,
    )
    .unwrap();
    assert_eq!(artifact.bytes, b"bonjour");
    assert_eq!(artifact.metadata.bytes, 7);
    assert_eq!(artifact.metadata.mime_type, "text/plain");
    assert_eq!(artifact.metadata.name, "résultat.txt");
    assert_eq!(artifact.metadata.purpose, ArtifactPurpose::Artifact);
    let super::super::agent_local::tool_artifact::ArtifactSource::WorkspaceFile { path, grant } =
        &artifact.metadata.source
    else {
        panic!("workspace source");
    };
    assert!(!grant.is_empty());
    assert_eq!(
        crate::services::attachment_access::verify_access_grant(
            path.to_str().unwrap(),
            grant,
            &KEY
        )
        .unwrap(),
        *path
    );
    let source = serde_json::to_value(&artifact.metadata).unwrap()["source"].clone();
    assert_eq!(source.as_object().unwrap().len(), 3);
    assert!(source.get("path").is_some());
    assert!(source.get("grant").is_some());
}

#[test]
fn declared_name_or_basename_is_visible_metadata() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("output.txt"), "x").unwrap();
    assert_eq!(
        read_workspace_file(
            root.path(),
            "output.txt",
            Some("Visible"),
            ArtifactPurpose::Artifact,
            &KEY,
        )
        .unwrap()
        .metadata
        .name,
        "Visible"
    );
}

#[test]
fn derived_unicode_name_is_safely_bounded_by_the_contract() {
    let root = tempfile::tempdir().unwrap();
    let name = format!(
        "{}{}.txt",
        "é".repeat(super::types::MAX_EXTENSION_NAME_CHARS),
        "fin"
    );
    std::fs::write(root.path().join(&name), "x").unwrap();

    let visible = read_workspace_file(root.path(), &name, None, ArtifactPurpose::Artifact, &KEY)
        .unwrap()
        .metadata
        .name;

    assert_eq!(
        visible.chars().count(),
        super::types::MAX_EXTENSION_NAME_CHARS
    );
    assert!(visible.is_char_boundary(visible.len()));
}

#[test]
fn signature_wins_over_misleading_extension() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("image.txt"), b"\x89PNG\r\n\x1a\nrest").unwrap();
    assert_eq!(
        read_workspace_file(
            root.path(),
            "image.txt",
            None,
            ArtifactPurpose::Artifact,
            &KEY,
        )
        .unwrap()
        .metadata
        .mime_type,
        "image/png"
    );
}

#[test]
fn absolute_traversal_and_directory_paths_are_refused() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("folder")).unwrap();
    let absolute = root.path().join("folder").to_string_lossy().to_string();

    for path in [&absolute, "../outside.txt", "folder"] {
        assert!(matches!(
            read_workspace_file(root.path(), path, None, ArtifactPurpose::Artifact, &KEY),
            Err(FileResultError::Access)
        ));
    }
}

#[test]
fn hard_links_are_refused() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original.txt");
    std::fs::write(&original, b"content").unwrap();
    std::fs::hard_link(&original, root.path().join("linked.txt")).unwrap();

    assert!(matches!(
        read_workspace_file(
            root.path(),
            "linked.txt",
            None,
            ArtifactPurpose::Artifact,
            &KEY,
        ),
        Err(FileResultError::Access)
    ));
}

#[cfg(unix)]
#[test]
fn symbolic_links_are_refused() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("target.txt"), b"content").unwrap();
    symlink("target.txt", root.path().join("linked.txt")).unwrap();

    assert!(matches!(
        read_workspace_file(
            root.path(),
            "linked.txt",
            None,
            ArtifactPurpose::Artifact,
            &KEY,
        ),
        Err(FileResultError::Access)
    ));
}

#[test]
fn missing_oversized_and_invalid_grant_inputs_are_classified() {
    let root = tempfile::tempdir().unwrap();
    assert!(matches!(
        read_workspace_file(
            root.path(),
            "missing.txt",
            None,
            ArtifactPurpose::Artifact,
            &KEY,
        ),
        Err(FileResultError::NotFound)
    ));

    let oversized = root.path().join("oversized.bin");
    let file = std::fs::File::create(&oversized).unwrap();
    file.set_len(super::types::MAX_RESULT_BYTES as u64 + 1)
        .unwrap();
    assert!(matches!(
        read_workspace_file(
            root.path(),
            "oversized.bin",
            None,
            ArtifactPurpose::Artifact,
            &KEY,
        ),
        Err(FileResultError::Limit)
    ));

    std::fs::write(root.path().join("valid.txt"), b"content").unwrap();
    assert!(matches!(
        read_workspace_file(
            root.path(),
            "valid.txt",
            None,
            ArtifactPurpose::Artifact,
            b"short",
        ),
        Err(FileResultError::Grant)
    ));
    assert!(matches!(
        read_workspace_file(
            root.path(),
            "valid.txt",
            Some(""),
            ArtifactPurpose::Artifact,
            &KEY,
        ),
        Err(FileResultError::Access)
    ));
}

#[cfg(unix)]
#[test]
fn fifo_is_refused_before_opening() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let root = tempfile::tempdir().unwrap();
    let fifo = root.path().join("pipe");
    let raw = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(raw.as_ptr(), 0o600) }, 0);

    assert!(matches!(
        read_workspace_file(root.path(), "pipe", None, ArtifactPurpose::Artifact, &KEY,),
        Err(FileResultError::Access)
    ));
}
