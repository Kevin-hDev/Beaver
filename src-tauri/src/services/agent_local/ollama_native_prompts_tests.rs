use super::ollama_native_prompts::{
    lookup_origin, NativePromptCatalog, NativePromptOrigin, NativePromptState, NativePromptStore,
};
use super::model_customizations::CustomizationKind;

#[test]
fn native_prompt_catalog_distinguishes_unknown_absent_and_present_models() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-native-system-prompts.json");
    let mut catalog = NativePromptCatalog::default();

    assert_eq!(catalog.get("legacy:latest"), None);
    catalog
        .record("gemma4:e2b", NativePromptState::Absent)
        .unwrap();
    catalog
        .record(
            "phi4-reasoning:latest",
            NativePromptState::Present("Native Phi prompt".into()),
        )
        .unwrap();
    catalog.write_to_path(&path).unwrap();

    let loaded = NativePromptCatalog::read_from_path(&path).unwrap();
    assert_eq!(loaded.get("gemma4:e2b"), Some(&NativePromptState::Absent));
    assert_eq!(
        loaded.get("phi4-reasoning:latest"),
        Some(&NativePromptState::Present("Native Phi prompt".into()))
    );
    assert_eq!(loaded.get("legacy:latest"), None);
}

#[test]
fn corrupt_native_prompt_catalog_is_reported_as_unavailable() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-native-system-prompts.json");
    std::fs::write(&path, b"{not valid json").unwrap();

    assert_eq!(
        NativePromptCatalog::read_from_path(&path).err(),
        Some("ollama-native-prompt-store-unavailable".to_string())
    );
}

#[cfg(unix)]
#[test]
fn native_prompt_catalog_rejects_symbolic_links() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("target.json");
    let path = directory.path().join("ollama-native-system-prompts.json");
    std::fs::write(&target, b"{\"models\":{}}").unwrap();
    symlink(target, &path).unwrap();

    assert!(NativePromptCatalog::read_from_path(&path).is_err());
}

#[test]
fn corrupt_native_prompt_store_is_never_overwritten() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-native-system-prompts.json");
    let corrupt = b"{not valid json";
    std::fs::write(&path, corrupt).unwrap();
    let store = NativePromptStore::open(path.clone());

    assert_eq!(
        store.record("gemma4:e2b", NativePromptState::Absent),
        Err("ollama-native-prompt-store-unavailable".to_string())
    );
    assert_eq!(std::fs::read(path).unwrap(), corrupt);
}

#[test]
fn unavailable_native_prompt_store_recovers_after_a_valid_file_is_restored() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-native-system-prompts.json");
    std::fs::write(&path, b"{not valid json").unwrap();
    let store = NativePromptStore::open(path.clone());
    NativePromptCatalog::default().write_to_path(&path).unwrap();

    store
        .record(
            "phi4:latest",
            NativePromptState::Present("native prompt".to_string()),
        )
        .unwrap();
    assert_eq!(
        store.cached("phi4:latest").unwrap(),
        Some(NativePromptState::Present("native prompt".to_string()))
    );
}

#[test]
fn deleted_native_prompt_store_reports_that_the_file_is_missing() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory
        .path()
        .join("ollama-native-system-prompts.json");
    let store = NativePromptStore::open(path.clone());
    store
        .record("first:latest", NativePromptState::Absent)
        .unwrap();
    std::fs::remove_file(path).unwrap();

    assert_eq!(
        store.record("second:latest", NativePromptState::Absent),
        Err("ollama-native-prompt-store-missing".to_string())
    );
}

#[test]
fn native_prompt_store_does_not_overwrite_corruption_that_happens_after_open() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-native-system-prompts.json");
    let store = NativePromptStore::open(path.clone());
    store.record("first:latest", NativePromptState::Absent).unwrap();
    let corrupt = b"{corrupted while Beaver is running";
    std::fs::write(&path, corrupt).unwrap();

    assert_eq!(
        store.record("second:latest", NativePromptState::Absent),
        Err("ollama-native-prompt-store-unavailable".to_string())
    );
    assert_eq!(std::fs::read(path).unwrap(), corrupt);
}

#[test]
fn native_prompt_origin_only_trusts_safe_local_sources() {
    assert_eq!(
        lookup_origin(Some(CustomizationKind::Unknown), false),
        NativePromptOrigin::Unknown
    );
    assert_eq!(
        lookup_origin(Some(CustomizationKind::Modelfile), false),
        NativePromptOrigin::Unknown
    );
    assert_eq!(
        lookup_origin(Some(CustomizationKind::ParametersOnly), false),
        NativePromptOrigin::CurrentModel
    );
    assert_eq!(lookup_origin(None, false), NativePromptOrigin::CurrentModel);
    assert_eq!(
        lookup_origin(Some(CustomizationKind::Unknown), true),
        NativePromptOrigin::Catalog
    );
}
