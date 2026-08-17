use crate::services::ollama_manager::{release_source::archive_names_for_platform, OllamaVersion};

#[test]
fn semver_rejects_path_traversal() {
    assert!(OllamaVersion::parse("1.0.0/../../evil").is_err());
    assert!(OllamaVersion::parse("1.0.0%0d%0a").is_err());
    assert!(OllamaVersion::parse("1.0.0\n").is_err());
    assert!(OllamaVersion::parse("").is_err());
    assert!(OllamaVersion::parse("abc").is_err());
    assert!(OllamaVersion::parse("1.0").is_err());
}

#[test]
fn semver_accepts_valid_versions() {
    assert!(OllamaVersion::parse("0.23.1").is_ok());
    assert!(OllamaVersion::parse("0.30.0-rc3").is_ok());
    assert!(OllamaVersion::parse("1.0.0-beta.1").is_ok());
}

#[test]
fn semver_rejects_v_prefix() {
    assert!(OllamaVersion::parse("v1.0.0").is_err());
}

#[test]
fn archives_returns_nonempty() {
    let a = archive_names_for_platform();
    assert!(!a.is_empty());
    for name in &a {
        assert!(
            name.ends_with(".tgz") || name.ends_with(".tar.zst") || name.ends_with(".zip"),
            "unexpected archive format: {name}"
        );
    }
}
