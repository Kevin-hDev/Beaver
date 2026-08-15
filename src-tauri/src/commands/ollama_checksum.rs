#![allow(dead_code)]

use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn fetch_expected_hash(version: &str, archive_name: &str) -> Result<String, String> {
    let version = crate::services::ollama_manager::OllamaVersion::parse(version)
        .map_err(|_| "checksum-not-available".to_string())?;
    let manifest =
        crate::services::ollama_manager::release_source::fetch_manifest(version, &[archive_name])
            .await
            .map_err(|_| "checksum-not-available".to_string())?;
    manifest
        .archives()
        .first()
        .map(|archive| archive.sha256.to_hex())
        .ok_or_else(|| "checksum-not-found".to_string())
}

fn parse_sha256_line(content: &str, archive_name: &str) -> Option<String> {
    let name = crate::services::ollama_manager::AllowlistedArchiveName::parse(archive_name).ok()?;
    crate::services::ollama_manager::release_source::parse_sha256_manifest(
        content.as_bytes(),
        &name,
    )
    .ok()
    .map(|digest| digest.to_hex())
}

pub fn verify_file_sha256(
    path: &Path,
    expected: &str,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err(super::ollama_setup_cancel::cancelled_error());
    }
    let expected = crate::services::ollama_manager::Sha256Digest::from_hex(expected)
        .map_err(|_| "checksum-mismatch".to_string())?;
    crate::services::ollama_manager::verify_sha256(path, &expected)
        .map_err(|_| "checksum-mismatch".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sha256_finds_correct_hash() {
        let content = "\
629586ff1a76d351a7b9f57eb1985ef903ee2e6ff4997a7c5573964c83f37a16  ./ollama-darwin.tgz
1079dad63e0e0f2d3279280ce6d8a93f3af53e79afb14f74813e0b35d9b96d54  ./ollama-linux-amd64.tar.zst
";
        let hash = parse_sha256_line(content, "ollama-darwin.tgz");
        assert_eq!(
            hash.unwrap(),
            "629586ff1a76d351a7b9f57eb1985ef903ee2e6ff4997a7c5573964c83f37a16"
        );
    }

    #[test]
    fn parse_sha256_returns_none_for_missing() {
        let content = "abc123  ./other-file.tgz\n";
        assert!(parse_sha256_line(content, "ollama-darwin.tgz").is_none());
    }

    #[test]
    fn parse_sha256_rejects_invalid_hash() {
        let content = "not-a-hash  ./ollama-darwin.tgz\n";
        assert!(parse_sha256_line(content, "ollama-darwin.tgz").is_none());
    }

    #[test]
    fn verify_file_sha256_correct() {
        let dir = std::env::temp_dir().join("cl-go-sha256-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.bin");
        std::fs::write(&path, b"hello world").unwrap();

        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let cancel = CancellationToken::new();
        assert!(verify_file_sha256(&path, expected, &cancel).is_ok());

        assert!(verify_file_sha256(
            &path,
            "0000000000000000000000000000000000000000000000000000000000000000",
            &cancel
        )
        .is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
