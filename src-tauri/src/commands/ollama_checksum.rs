use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use tokio_util::sync::CancellationToken;

use crate::services::brand;

const CHECKSUM_TIMEOUT_SECS: u64 = 15;
const MAX_CHECKSUM_BYTES: usize = 8 * 1024;

pub async fn fetch_expected_hash(version: &str, archive_name: &str) -> Result<String, String> {
    let url = format!(
        "https://github.com/ollama/ollama/releases/download/v{}/sha256sum.txt",
        version
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(CHECKSUM_TIMEOUT_SECS))
        .build()
        .map_err(|_| "checksum-client-error".to_string())?;

    let resp = client
        .get(&url)
        .header("User-Agent", brand::user_agent())
        .send()
        .await
        .map_err(|_| "checksum-download-error".to_string())?;

    if !resp.status().is_success() {
        return Err("checksum-not-available".into());
    }

    let body = resp
        .text()
        .await
        .map_err(|_| "checksum-parse-error".to_string())?;

    if body.len() > MAX_CHECKSUM_BYTES {
        return Err("checksum-file-too-large".into());
    }

    parse_sha256_line(&body, archive_name).ok_or_else(|| "checksum-not-found".into())
}

fn parse_sha256_line(content: &str, archive_name: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() != 2 {
            continue;
        }
        let hash = parts[0].trim();
        let name = parts[1].trim().trim_start_matches("./");
        if name == archive_name && hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(hash.to_lowercase());
        }
    }
    None
}

pub fn verify_file_sha256(
    path: &Path,
    expected: &str,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let mut file = std::fs::File::open(path).map_err(|e| {
        ::log::warn!("[ollama-checksum] open: {e}");
        "checksum-read-error".to_string()
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        if cancel.is_cancelled() {
            return Err(super::ollama_setup_cancel::cancelled_error());
        }
        let read = file.read(&mut buffer).map_err(|e| {
            ::log::warn!("[ollama-checksum] read: {e}");
            "checksum-read-error".to_string()
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let actual = format!("{:x}", hasher.finalize());

    if actual != expected {
        ::log::warn!(
            "[ollama-checksum] mismatch: expected={} actual={}",
            &expected[..12],
            &actual[..12]
        );
        return Err("checksum-mismatch".into());
    }

    Ok(())
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
