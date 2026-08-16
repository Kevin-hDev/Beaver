use super::error::OllamaErrorCode;
use super::extract::{
    extract_archive, validate_empty_staging, validate_member_path, ArchiveMemberKind,
};
use super::extract_fixture::{
    empty_staging, raw_header, write_duplicate_zip, write_gzip_tar, write_symlink_zip, write_zip,
    TarMember, ZipMember,
};
use std::io::Write;
use std::path::Path;
use tokio_util::sync::CancellationToken;

#[test]
fn extraction_rejects_traversal_and_absolute_paths() {
    for path in ["../escape", "a/../../escape", "/absolute", "\\absolute"] {
        assert!(validate_member_path(Path::new(path)).is_err(), "{path}");
    }
}

#[test]
fn extraction_rejects_links_and_duplicate_members() {
    assert_eq!(
        ArchiveMemberKind::Symlink.validate(),
        Err(OllamaErrorCode::OllamaExtractionFailed)
    );
    assert_eq!(
        ArchiveMemberKind::Hardlink.validate(),
        Err(OllamaErrorCode::OllamaExtractionFailed)
    );
}

#[test]
fn extraction_requires_an_existing_empty_regular_staging_directory() {
    let root = tempfile::tempdir().unwrap();
    let staging = empty_staging(root.path());
    assert!(validate_empty_staging(&staging).is_ok());
    std::fs::write(staging.join("file"), b"x").unwrap();
    assert!(validate_empty_staging(&staging).is_err());
}

#[cfg(unix)]
#[test]
fn extraction_rejects_parent_swap_after_validation_before_write() {
    let root = tempfile::tempdir().unwrap();
    let archive = root.path().join("swap.tgz");
    write_gzip_tar(
        &archive,
        &[TarMember::regular(
            "bin/ollama",
            b"must-stay-confined",
            0o755,
        )],
    );
    let staging = empty_staging(root.path());
    let outside = root.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    let swapped_parent = staging.join("bin");
    let result = super::extract::extract_archive_for_test(
        &archive,
        &staging,
        "ollama-darwin.tgz",
        &CancellationToken::new(),
        || {
            std::os::unix::fs::symlink(&outside, &swapped_parent).unwrap();
            Ok(())
        },
    );
    assert_eq!(result, Err(OllamaErrorCode::OllamaBundleInvalid));
    assert!(!outside.join("ollama").exists());
}

#[test]
fn real_tar_and_zip_archives_extract_regular_files() {
    let root = tempfile::tempdir().unwrap();
    let tar_path = root.path().join("ollama-darwin.tgz");
    write_gzip_tar(
        &tar_path,
        &[TarMember::regular("bin/ollama", b"tar-binary", 0o755)],
    );
    let tar_staging = empty_staging(root.path());
    extract_archive(
        &tar_path,
        &tar_staging,
        "ollama-darwin.tgz",
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(
        std::fs::read(tar_staging.join("bin/ollama")).unwrap(),
        b"tar-binary"
    );
    #[cfg(unix)]
    assert!(super::extract_fixture::is_executable(
        &tar_staging.join("bin/ollama")
    ));

    let zip_path = root.path().join("ollama-windows-amd64.zip");
    write_zip(
        &zip_path,
        &[ZipMember::regular("bin/ollama.exe", b"zip-binary")],
    );
    let zip_staging = root.path().join("zip-staging");
    std::fs::create_dir(&zip_staging).unwrap();
    extract_archive(
        &zip_path,
        &zip_staging,
        "ollama-windows-amd64.zip",
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(
        std::fs::read(zip_staging.join("bin/ollama.exe")).unwrap(),
        b"zip-binary"
    );
}

#[cfg(unix)]
#[test]
fn official_unix_archive_extracts_only_contained_relative_symlink_chains() {
    let root = tempfile::tempdir().unwrap();
    let archive = root.path().join("ollama-darwin.tgz");
    write_gzip_tar(
        &archive,
        &[
            TarMember::regular("libggml.0.19.0.dylib", b"library", 0o755),
            TarMember::raw(
                "libggml.0.dylib",
                tar::EntryType::Symlink,
                &[],
                0o777,
                Some("libggml.0.19.0.dylib"),
            ),
            TarMember::raw(
                "libggml.dylib",
                tar::EntryType::Symlink,
                &[],
                0o777,
                Some("libggml.0.dylib"),
            ),
        ],
    );
    let staging = empty_staging(root.path());

    extract_archive(
        &archive,
        &staging,
        "ollama-darwin.tgz",
        &CancellationToken::new(),
    )
    .unwrap();

    assert_eq!(
        std::fs::read_link(staging.join("libggml.0.dylib")).unwrap(),
        Path::new("libggml.0.19.0.dylib")
    );
    assert_eq!(
        std::fs::read_link(staging.join("libggml.dylib")).unwrap(),
        Path::new("libggml.0.dylib")
    );
    assert_eq!(
        std::fs::read(staging.join("libggml.dylib")).unwrap(),
        b"library"
    );
}

#[test]
fn tar_traversal_absolute_symlink_and_hardlink_are_rejected_without_escape() {
    let cases = [
        TarMember::raw("../escape", tar::EntryType::Regular, b"escape", 0o644, None),
        TarMember::raw(
            "/absolute",
            tar::EntryType::Regular,
            b"absolute",
            0o644,
            None,
        ),
        TarMember::raw(
            "bin/link",
            tar::EntryType::Symlink,
            &[],
            0o777,
            Some("/etc/passwd"),
        ),
        TarMember::raw(
            "bin/hardlink",
            tar::EntryType::Link,
            &[],
            0o644,
            Some("bin/ollama"),
        ),
    ];
    for (index, member) in cases.into_iter().enumerate() {
        let root = tempfile::tempdir().unwrap();
        let archive = root.path().join(format!("case-{index}.tgz"));
        write_gzip_tar(&archive, &[member]);
        let staging = empty_staging(root.path());
        assert!(
            extract_archive(
                &archive,
                &staging,
                "ollama-darwin.tgz",
                &CancellationToken::new(),
            )
            .is_err(),
            "case {index}"
        );
        assert!(!root.path().join("escape").exists());
        assert!(!root.path().join("absolute").exists());
    }
}

#[cfg(unix)]
#[test]
fn unix_archive_rejects_parent_missing_and_forward_symlink_targets() {
    let cases = [
        vec![TarMember::raw(
            "bin/link",
            tar::EntryType::Symlink,
            &[],
            0o777,
            Some("../outside"),
        )],
        vec![TarMember::raw(
            "libmissing.dylib",
            tar::EntryType::Symlink,
            &[],
            0o777,
            Some("missing.dylib"),
        )],
        vec![
            TarMember::raw(
                "libforward.dylib",
                tar::EntryType::Symlink,
                &[],
                0o777,
                Some("libtarget.dylib"),
            ),
            TarMember::regular("libtarget.dylib", b"library", 0o755),
        ],
    ];
    for (index, members) in cases.into_iter().enumerate() {
        let root = tempfile::tempdir().unwrap();
        let archive = root.path().join(format!("unsafe-link-{index}.tgz"));
        write_gzip_tar(&archive, &members);
        let staging = empty_staging(root.path());

        assert_eq!(
            extract_archive(
                &archive,
                &staging,
                "ollama-darwin.tgz",
                &CancellationToken::new(),
            ),
            Err(OllamaErrorCode::OllamaBundleInvalid),
            "case {index}"
        );
        assert!(!root.path().join("outside").exists());
    }
}

#[test]
fn zip_traversal_absolute_and_symlink_are_rejected() {
    let cases = [
        ZipMember::regular("../escape", b"escape"),
        ZipMember::regular("/absolute", b"absolute"),
    ];
    for (index, member) in cases.into_iter().enumerate() {
        let root = tempfile::tempdir().unwrap();
        let archive = root.path().join(format!("case-{index}.zip"));
        write_zip(&archive, &[member]);
        let staging = empty_staging(root.path());
        assert!(
            extract_archive(
                &archive,
                &staging,
                "ollama-windows-amd64.zip",
                &CancellationToken::new(),
            )
            .is_err(),
            "case {index}"
        );
        assert!(!root.path().join("escape").exists());
        assert!(!root.path().join("absolute").exists());
    }
    let root = tempfile::tempdir().unwrap();
    let archive = root.path().join("symlink.zip");
    write_symlink_zip(&archive, "bin/link", b"/etc/passwd");
    let staging = empty_staging(root.path());
    assert_eq!(
        extract_archive(
            &archive,
            &staging,
            "ollama-windows-amd64.zip",
            &CancellationToken::new()
        ),
        Err(OllamaErrorCode::OllamaExtractionFailed)
    );
}

#[test]
fn tar_and_zip_duplicate_members_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let tar_path = root.path().join("duplicate.tgz");
    write_gzip_tar(
        &tar_path,
        &[
            TarMember::regular("bin/ollama", b"first", 0o755),
            TarMember::regular("bin/ollama", b"second", 0o755),
        ],
    );
    let staging = empty_staging(root.path());
    assert_eq!(
        extract_archive(
            &tar_path,
            &staging,
            "ollama-darwin.tgz",
            &CancellationToken::new()
        ),
        Err(OllamaErrorCode::OllamaBundleInvalid)
    );

    let zip_path = root.path().join("duplicate.zip");
    write_duplicate_zip(&zip_path, "bin/ollama.exe", b"first", b"second");
    let zip_staging = root.path().join("zip-staging");
    std::fs::create_dir(&zip_staging).unwrap();
    assert_eq!(
        extract_archive(
            &zip_path,
            &zip_staging,
            "ollama-windows-amd64.zip",
            &CancellationToken::new()
        ),
        Err(OllamaErrorCode::OllamaBundleInvalid)
    );
}

#[test]
fn oversized_tar_member_is_rejected_before_copying_the_flux() {
    let root = tempfile::tempdir().unwrap();
    let archive = root.path().join("oversized.tgz");
    let header = raw_header(
        "bin/ollama",
        tar::EntryType::Regular,
        4 * 1024 * 1024 * 1024 + 1,
        0o755,
        None,
    );
    let mut raw = Vec::from(header.as_bytes());
    raw.extend_from_slice(&[0; 1024]);
    let file = std::fs::File::create(&archive).unwrap();
    let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    encoder.write_all(&raw).unwrap();
    encoder.finish().unwrap();
    let staging = empty_staging(root.path());
    assert_eq!(
        extract_archive(
            &archive,
            &staging,
            "ollama-darwin.tgz",
            &CancellationToken::new()
        ),
        Err(OllamaErrorCode::OllamaBundleInvalid)
    );
    assert!(!staging.join("bin/ollama").exists());
}
