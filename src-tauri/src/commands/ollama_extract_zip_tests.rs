use crate::commands::ollama_extract_zip::extract_zip;
use std::io::Write;
use tokio_util::sync::CancellationToken;

#[test]
fn extract_zip_rejects_parent_dir() {
    let dir = std::env::temp_dir().join("cl-go-test-zip-traversal");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let zip_path = dir.join("evil.zip");
    write_zip_entry(&zip_path, "../escape.txt", b"pwned");
    let dest = dir.join("output");
    std::fs::create_dir_all(&dest).unwrap();

    let result = extract_zip(&zip_path, &dest, &CancellationToken::new());

    assert!(result.is_err(), "should reject zip path traversal");
    assert!(
        !dir.join("escape.txt").exists(),
        "file should not escape dest"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn extract_zip_respects_cancelled_token() {
    let dir = std::env::temp_dir().join("cl-go-test-zip-cancel");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let zip_path = dir.join("good.zip");
    write_zip_entry(&zip_path, "bin/ollama.exe", b"binary");
    let dest = dir.join("output");
    std::fs::create_dir_all(&dest).unwrap();

    let cancel = CancellationToken::new();
    cancel.cancel();
    let result = extract_zip(&zip_path, &dest, &cancel);

    assert!(
        result.is_err(),
        "cancelled install should stop zip extraction"
    );
    assert!(!dest.join("bin/ollama.exe").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

fn write_zip_entry(path: &std::path::Path, name: &str, data: &[u8]) {
    let file = std::fs::File::create(path).unwrap();
    let mut archive = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    archive.start_file(name, options).unwrap();
    archive.write_all(data).unwrap();
    archive.finish().unwrap();
}
