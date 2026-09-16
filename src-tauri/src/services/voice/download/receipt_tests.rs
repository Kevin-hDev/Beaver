use super::{
    receipt::{load_receipt, save_receipt, InstallationReceipt},
    VoiceArchive, VoiceCatalogEntry, VoiceEngine, VoiceLanguageMode, VoiceLicense, VoiceModelFile,
    VoiceModelRole,
};

pub(super) fn raw_entry(bytes: &[u8]) -> VoiceCatalogEntry {
    let digest = hex::encode(sha2::Sha256::digest(bytes));
    VoiceCatalogEntry {
        id: "silero-vad".into(),
        role: VoiceModelRole::Vad,
        engine: VoiceEngine::SileroVad,
        revision: format!("sha256:{digest}"),
        archive: VoiceArchive {
            url: "https://github.com/test".into(),
            bytes: bytes.len() as u64,
            sha256: digest.clone(),
        },
        manifest_url: "https://github.com/test".into(),
        installed_bytes: bytes.len() as u64,
        files: vec![VoiceModelFile {
            path: "silero_vad.onnx".into(),
            bytes: bytes.len() as u64,
            sha256: digest,
        }],
        languages: vec![],
        dialects: vec![],
        language_mode: VoiceLanguageMode::AutomaticOnly,
        license: VoiceLicense {
            spdx: "MIT".into(),
            url: "https://github.com/test".into(),
        },
    }
}

use sha2::Digest;

#[test]
fn receipt_round_trip_preserves_the_exact_manifest() {
    let data = tempfile::tempdir().unwrap();
    let receipt = InstallationReceipt::from_entry(&raw_entry(b"vad"));
    save_receipt(data.path(), &receipt).unwrap();
    assert_eq!(
        load_receipt(data.path(), "silero-vad").unwrap(),
        Some(receipt)
    );
}

#[test]
fn sha256_revision_keeps_its_receipt_but_uses_a_portable_directory_name() {
    let data = tempfile::tempdir().unwrap();
    let receipt = InstallationReceipt::from_entry(&raw_entry(b"vad"));

    assert!(receipt.revision.starts_with("sha256:"));
    assert_eq!(
        receipt.install_dir(data.path()).file_name().unwrap(),
        receipt.revision.strip_prefix("sha256:").unwrap()
    );
}
