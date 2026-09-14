use std::fs;

use serde_json::Value;

use super::load_catalog;

fn source_catalog() -> Value {
    serde_json::from_str(include_str!("../../../../resources/voice-catalog.json"))
        .expect("source catalogue")
}

fn write_catalog(value: &Value) -> tempfile::TempDir {
    let resources = tempfile::tempdir().expect("test resources");
    let voice_resources = resources.path().join("resources");
    fs::create_dir(&voice_resources).expect("voice resource directory");
    fs::write(
        voice_resources.join("voice-catalog.json"),
        serde_json::to_vec(value).expect("catalogue JSON"),
    )
    .expect("catalogue fixture");
    resources
}

fn rejects(mutator: impl FnOnce(&mut Value)) {
    let mut value = source_catalog();
    mutator(&mut value);
    let resources = write_catalog(&value);
    assert!(load_catalog(resources.path()).is_err());
}

#[test]
fn source_catalog_is_complete_and_valid() {
    let resources = write_catalog(&source_catalog());
    let catalog = load_catalog(resources.path()).expect("valid catalogue");
    assert_eq!(catalog.entries.len(), 4);
    let qwen = catalog
        .entries
        .iter()
        .find(|entry| entry.id == "qwen3-asr-06b-int8")
        .expect("Qwen entry");
    assert_eq!(qwen.languages.len(), 30);
    assert_eq!(qwen.dialects.len(), 22);
}

#[test]
fn missing_catalog_fails_closed() {
    let resources = tempfile::tempdir().expect("test resources");
    assert!(load_catalog(resources.path()).is_err());
}

#[test]
fn duplicate_entry_is_rejected() {
    rejects(|value| {
        let entries = value["entries"].as_array_mut().expect("entries");
        entries.push(entries[0].clone());
    });
}

#[test]
fn traversing_identifier_is_rejected() {
    rejects(|value| value["entries"][0]["id"] = "../model".into());
}

#[test]
fn unsafe_sources_are_rejected() {
    rejects(|value| value["entries"][0]["archive"]["url"] = "http://example.com/a".into());
    rejects(|value| value["entries"][0]["manifest_url"] = "https://untrusted.example/a".into());
}

#[test]
fn absent_hash_is_rejected() {
    rejects(|value| value["entries"][0]["files"][0]["sha256"] = "".into());
}

#[test]
fn catalogue_limits_are_enforced() {
    rejects(|value| {
        let entries = value["entries"].as_array_mut().expect("entries");
        while entries.len() <= super::super::limits::MAX_CATALOG_ENTRIES {
            let mut copy = entries[0].clone();
            copy["id"] = format!("model-{}", entries.len()).into();
            entries.push(copy);
        }
    });
}

#[test]
fn required_file_is_enforced() {
    rejects(|value| {
        value["entries"][0]["files"]
            .as_array_mut()
            .expect("files")
            .retain(|file| file["path"] != "tokens.txt");
    });
}

#[test]
fn invalid_language_is_rejected() {
    rejects(|value| value["entries"][0]["languages"][0] = "fr_FR".into());
}
