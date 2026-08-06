use super::ollama_native_prompts::{
    lookup_origin, parse_native_layer, registry_model_path, NativeLayer, NativePromptCatalog,
    NativePromptOrigin, NativePromptState,
};

#[test]
fn registry_without_system_layer_means_no_native_prompt() {
    let manifest = br#"{
      "schemaVersion": 2,
      "layers": [
        {"mediaType":"application/vnd.ollama.image.model","digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","size":42}
      ]
    }"#;

    assert_eq!(parse_native_layer(manifest).unwrap(), NativeLayer::Absent);
}

#[test]
fn registry_system_layer_identifies_the_downloaded_native_prompt() {
    let manifest = br#"{
      "schemaVersion": 2,
      "layers": [
        {"mediaType":"application/vnd.ollama.image.system","digest":"sha256:88df15fe1f347e9837ab4579c60f4fbae1cd1abf5a5ceab1bd93b846f65e1228","size":1232}
      ]
    }"#;

    assert_eq!(
        parse_native_layer(manifest).unwrap(),
        NativeLayer::Present {
            digest: "sha256:88df15fe1f347e9837ab4579c60f4fbae1cd1abf5a5ceab1bd93b846f65e1228".into(),
            size: 1232,
        }
    );
}

#[test]
fn registry_path_accepts_library_and_namespaced_models_only() {
    assert_eq!(
        registry_model_path("gemma4:e2b"),
        Some(("library/gemma4".into(), "e2b".into()))
    );
    assert_eq!(
        registry_model_path("acme/model:latest"),
        Some(("acme/model".into(), "latest".into()))
    );
    assert_eq!(registry_model_path("other.example/acme/model:latest"), None);
}

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
fn customized_legacy_model_is_verified_against_registry_not_current_modelfile() {
    assert_eq!(lookup_origin(true, false), NativePromptOrigin::Registry);
    assert_eq!(lookup_origin(false, false), NativePromptOrigin::CurrentModel);
    assert_eq!(lookup_origin(true, true), NativePromptOrigin::Catalog);
}
