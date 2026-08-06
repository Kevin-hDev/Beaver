use super::ollama_native_prompts::{
    lookup_origin, NativePromptCatalog, NativePromptOrigin, NativePromptState,
};

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

    let loaded = NativePromptCatalog::read_from_path(&path);
    assert_eq!(loaded.get("gemma4:e2b"), Some(&NativePromptState::Absent));
    assert_eq!(
        loaded.get("phi4-reasoning:latest"),
        Some(&NativePromptState::Present("Native Phi prompt".into()))
    );
    assert_eq!(loaded.get("legacy:latest"), None);
}

#[test]
fn customized_legacy_model_without_capture_stays_unavailable_offline() {
    assert_eq!(lookup_origin(true, false), NativePromptOrigin::Unavailable);
    assert_eq!(lookup_origin(false, false), NativePromptOrigin::CurrentModel);
    assert_eq!(lookup_origin(true, true), NativePromptOrigin::Catalog);
}
