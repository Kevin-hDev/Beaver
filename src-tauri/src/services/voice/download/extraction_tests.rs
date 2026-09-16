use std::io::Write;

use sha2::{Digest, Sha256};

use super::{
    extraction::extract_verified, VoiceArchive, VoiceCatalogEntry, VoiceEngine, VoiceLanguageMode,
    VoiceLicense, VoiceModelFile, VoiceModelRole,
};

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn archive_with_file(path: &str, data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    {
        let encoder = bzip2::write::BzEncoder::new(&mut compressed, bzip2::Compression::best());
        let mut archive = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o600);
        header.set_cksum();
        archive.append_data(&mut header, path, data).unwrap();
        archive.into_inner().unwrap().finish().unwrap();
    }
    compressed
}

fn entry(archive: &[u8], model: &[u8]) -> VoiceCatalogEntry {
    VoiceCatalogEntry {
        id: "test-model".into(),
        role: VoiceModelRole::Asr,
        engine: VoiceEngine::NemoTransducer,
        revision: "a".repeat(40),
        archive: VoiceArchive {
            url: "https://github.com/test".into(),
            bytes: archive.len() as u64,
            sha256: sha(archive),
        },
        manifest_url: "https://github.com/test".into(),
        installed_bytes: model.len() as u64,
        files: vec![VoiceModelFile {
            path: "model.onnx".into(),
            bytes: model.len() as u64,
            sha256: sha(model),
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

#[test]
fn extracts_only_the_verified_manifest_file() {
    let model = b"verified model";
    let archive = archive_with_file("bundle/model.onnx", model);
    let temp = tempfile::tempdir().unwrap();
    let archive_path = temp.path().join("model.tar.bz2");
    std::fs::write(&archive_path, &archive).unwrap();
    let destination = temp.path().join("out");

    extract_verified(&entry(&archive, model), &archive_path, &destination).unwrap();
    assert_eq!(
        std::fs::read(destination.join("model.onnx")).unwrap(),
        model
    );
}

#[test]
fn refuses_links_even_when_their_name_matches_the_manifest() {
    let mut compressed = Vec::new();
    {
        let encoder = bzip2::write::BzEncoder::new(&mut compressed, bzip2::Compression::best());
        let mut archive = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        header.set_link_name("/tmp/outside").unwrap();
        header.set_cksum();
        archive
            .append_data(&mut header, "bundle/model.onnx", std::io::empty())
            .unwrap();
        archive.into_inner().unwrap().finish().unwrap();
    }
    let temp = tempfile::tempdir().unwrap();
    let archive_path = temp.path().join("bad.tar.bz2");
    std::fs::File::create(&archive_path)
        .unwrap()
        .write_all(&compressed)
        .unwrap();

    assert_eq!(
        extract_verified(
            &entry(&compressed, b""),
            &archive_path,
            &temp.path().join("out")
        )
        .unwrap_err(),
        "model-download-archive-invalid"
    );
}
