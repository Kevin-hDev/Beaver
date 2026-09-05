use super::ui_protocol::{serve, ProofArtifact};
use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::http::{Method, Request, StatusCode};

fn fixture(root: &Path, bytes: &[u8]) -> ProofArtifact {
    let hash = format!("{:x}", Sha256::digest(bytes));
    let directory = root.join("ui-proof").join(&hash);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("entry.mjs"), bytes).unwrap();
    ProofArtifact::for_test(root.to_path_buf(), "ui-proof", &hash)
}

fn request(uri: &str, origin: &str) -> Request<Vec<u8>> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Origin", origin)
        .body(Vec::new())
        .unwrap()
}

fn request_without_origin(uri: &str) -> Request<Vec<u8>> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Vec::new())
        .unwrap()
}

#[test]
fn protocol_serves_only_the_allowlisted_fixture_with_closed_headers() {
    let root = tempfile::tempdir().unwrap();
    let artifact = fixture(root.path(), b"export const proof = 'loaded';");
    let uri = format!(
        "beaver-extension://localhost/ui-proof/{}/entry.mjs",
        artifact.manifest_sha()
    );

    let response = serve("main", &request(&uri, "tauri://localhost"), Some(&artifact));

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.body(), b"export const proof = 'loaded';");
    assert_eq!(
        response.headers()["Content-Type"],
        "text/javascript; charset=utf-8"
    );
    assert_eq!(response.headers()["X-Content-Type-Options"], "nosniff");
    assert_eq!(response.headers()["Cache-Control"], "no-store");
    assert_eq!(
        response.headers()["Access-Control-Allow-Origin"],
        "tauri://localhost"
    );
    let response = serve("main", &request_without_origin(&uri), Some(&artifact));
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["Access-Control-Allow-Origin"],
        "tauri://localhost"
    );
}

#[test]
fn protocol_refuses_same_size_fixture_tampering() {
    let root = tempfile::tempdir().unwrap();
    let original = b"export default true;";
    let replacement = b"export default null;";
    assert_eq!(original.len(), replacement.len());
    let artifact = fixture(root.path(), original);
    let directory = root.path().join("ui-proof").join(artifact.manifest_sha());
    std::fs::write(directory.join("entry.mjs"), replacement).unwrap();
    let uri = format!(
        "beaver-extension://localhost/ui-proof/{}/entry.mjs",
        artifact.manifest_sha()
    );

    let response = serve("main", &request(&uri, "tauri://localhost"), Some(&artifact));

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn protocol_refuses_other_webviews_origins_methods_and_hashes() {
    let root = tempfile::tempdir().unwrap();
    let artifact = fixture(root.path(), b"export default true;");
    let uri = format!(
        "http://beaver-extension.localhost/ui-proof/{}/entry.mjs",
        artifact.manifest_sha()
    );

    assert_eq!(
        serve(
            "other",
            &request(&uri, "http://tauri.localhost"),
            Some(&artifact)
        )
        .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        serve(
            "main",
            &request(&uri, "https://attacker.invalid"),
            Some(&artifact)
        )
        .status(),
        StatusCode::NOT_FOUND
    );
    let post = Request::builder()
        .method(Method::POST)
        .uri(&uri)
        .header("Origin", "http://tauri.localhost")
        .body(Vec::new())
        .unwrap();
    assert_eq!(
        serve("main", &post, Some(&artifact)).status(),
        StatusCode::NOT_FOUND
    );
    let bad_hash = "0".repeat(64);
    let uri = format!("beaver-extension://localhost/ui-proof/{bad_hash}/entry.mjs");
    assert_eq!(
        serve("main", &request(&uri, "tauri://localhost"), Some(&artifact)).status(),
        StatusCode::NOT_FOUND
    );
}

#[test]
fn protocol_refuses_traversal_symlinks_and_large_files() {
    let root = tempfile::tempdir().unwrap();
    let artifact = fixture(root.path(), b"export default true;");
    let prefix = format!(
        "beaver-extension://localhost/ui-proof/{}",
        artifact.manifest_sha()
    );
    assert_eq!(
        serve(
            "main",
            &request(&format!("{prefix}/../entry.mjs"), "tauri://localhost"),
            Some(&artifact)
        )
        .status(),
        StatusCode::NOT_FOUND
    );

    let directory = root.path().join("ui-proof").join(artifact.manifest_sha());
    std::fs::remove_file(directory.join("entry.mjs")).unwrap();
    std::fs::write(root.path().join("outside.mjs"), b"export default false;").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.path().join("outside.mjs"), directory.join("entry.mjs"))
            .unwrap();
        assert_eq!(
            serve(
                "main",
                &request(&format!("{prefix}/entry.mjs"), "tauri://localhost"),
                Some(&artifact)
            )
            .status(),
            StatusCode::NOT_FOUND
        );
        std::fs::remove_file(directory.join("entry.mjs")).unwrap();
    }
    std::fs::write(directory.join("entry.mjs"), vec![b'x'; 4_194_305]).unwrap();
    assert_eq!(
        serve(
            "main",
            &request(&format!("{prefix}/entry.mjs"), "tauri://localhost"),
            Some(&artifact)
        )
        .status(),
        StatusCode::NOT_FOUND
    );
}

#[test]
fn protocol_refuses_artifacts_in_safe_mode() {
    let root = tempfile::tempdir().unwrap();
    let artifact = fixture(root.path(), b"export default true;");
    let uri = format!(
        "beaver-extension://localhost/ui-proof/{}/entry.mjs",
        artifact.manifest_sha()
    );
    let startup = super::UiStartupState::resolved(super::UiStartupMode::Safe {
        reason: super::SafeReason::Argument,
    });

    assert_eq!(
        super::ui_protocol::serve_with_startup(
            "main",
            &request(&uri, "tauri://localhost"),
            &startup,
            Some(&artifact),
        )
        .status(),
        StatusCode::NOT_FOUND,
    );
}
