use super::model_customizations::{
    customization_kind_from, CustomizationKind, ModelCustomizationCatalog,
    ModelCustomizationStore,
};
use std::sync::{Arc, Barrier};

#[test]
fn legacy_model_names_are_migrated_as_unknown_customizations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    std::fs::write(&path, r#"{"models":["gemma4:e2b"]}"#).unwrap();

    let catalog = ModelCustomizationCatalog::read_from_path(&path).unwrap();

    assert_eq!(
        catalog.kind("gemma4:e2b"),
        Some(CustomizationKind::Unknown)
    );
}

#[test]
fn parameter_only_customization_remains_safe_to_capture_later() {
    let mut catalog = ModelCustomizationCatalog::default();

    catalog.mark_parameters("gemma4:e2b").unwrap();
    catalog.mark_parameters("gemma4:e2b").unwrap();

    assert_eq!(
        catalog.kind("gemma4:e2b"),
        Some(CustomizationKind::ParametersOnly)
    );
}

#[test]
fn parameter_update_does_not_make_legacy_or_modelfile_prompts_trustworthy() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    std::fs::write(&path, r#"{"models":["legacy:latest"]}"#).unwrap();
    let mut catalog = ModelCustomizationCatalog::read_from_path(&path).unwrap();
    catalog.mark_modelfile("edited:latest").unwrap();

    catalog.mark_parameters("legacy:latest").unwrap();
    catalog.mark_parameters("edited:latest").unwrap();

    assert_eq!(
        catalog.kind("legacy:latest"),
        Some(CustomizationKind::Unknown)
    );
    assert_eq!(
        catalog.kind("edited:latest"),
        Some(CustomizationKind::Modelfile)
    );
}

#[test]
fn customization_kinds_round_trip_without_losing_their_meaning() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    let mut catalog = ModelCustomizationCatalog::default();
    catalog.mark_parameters("parameters:latest").unwrap();
    catalog.mark_modelfile("modelfile:latest").unwrap();

    catalog.write_to_path(&path).unwrap();
    let loaded = ModelCustomizationCatalog::read_from_path(&path).unwrap();

    assert_eq!(
        loaded.kind("parameters:latest"),
        Some(CustomizationKind::ParametersOnly)
    );
    assert_eq!(
        loaded.kind("modelfile:latest"),
        Some(CustomizationKind::Modelfile)
    );
}

#[test]
fn opening_a_legacy_store_persists_the_migrated_format_once() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    std::fs::write(&path, r#"{"models":["legacy:latest"]}"#).unwrap();

    let store = ModelCustomizationStore::open(path.clone());

    assert_eq!(
        store.kind("legacy:latest").unwrap(),
        Some(CustomizationKind::Unknown)
    );
    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(persisted["models"]["legacy:latest"], "unknown");
}

#[test]
fn concurrent_customization_updates_do_not_overwrite_each_other() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    let store = Arc::new(ModelCustomizationStore::open(path.clone()));
    let barrier = Arc::new(Barrier::new(17));
    let mut workers = Vec::new();

    for index in 0..16 {
        let store = Arc::clone(&store);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            store
                .mark_parameters(&format!("concurrent-{index}:latest"))
                .unwrap();
        }));
    }
    barrier.wait();
    for worker in workers {
        worker.join().unwrap();
    }

    let persisted = ModelCustomizationCatalog::read_from_path(&path).unwrap();
    for index in 0..16 {
        assert_eq!(
            persisted.kind(&format!("concurrent-{index}:latest")),
            Some(CustomizationKind::ParametersOnly)
        );
    }
}

#[cfg(unix)]
#[test]
fn customization_catalog_is_written_with_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    let mut catalog = ModelCustomizationCatalog::default();
    catalog.mark_parameters("private:latest").unwrap();

    catalog.write_to_path(&path).unwrap();

    let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn maximum_customization_catalog_can_be_read_back() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    let mut catalog = ModelCustomizationCatalog::default();
    let suffix = "x".repeat(170);
    for index in 0..512 {
        catalog
            .mark_parameters(&format!("model-{index}-{suffix}"))
            .unwrap();
    }

    catalog.write_to_path(&path).unwrap();
    let loaded = ModelCustomizationCatalog::read_from_path(&path).unwrap();
    assert_eq!(
        loaded.kind(&format!("model-0-{suffix}")),
        Some(CustomizationKind::ParametersOnly)
    );
    assert_eq!(
        loaded.kind(&format!("model-511-{suffix}")),
        Some(CustomizationKind::ParametersOnly)
    );
}

#[test]
fn missing_customization_store_is_empty_and_can_be_created() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    let store = ModelCustomizationStore::open(path.clone());

    assert_eq!(customization_kind_from(&store, "new:latest"), None);
    store.mark_parameters("new:latest").unwrap();

    assert_eq!(
        customization_kind_from(&store, "new:latest"),
        Some(CustomizationKind::ParametersOnly)
    );
    assert!(path.exists());
}

#[test]
fn corrupt_customization_store_is_unknown_and_never_overwritten() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    let corrupt = b"{not valid json";
    std::fs::write(&path, corrupt).unwrap();
    let store = ModelCustomizationStore::open(path.clone());

    assert_eq!(
        customization_kind_from(&store, "legacy:latest"),
        Some(CustomizationKind::Unknown)
    );
    assert_eq!(
        store.mark_parameters("legacy:latest"),
        Err("ollama-custom-store-unavailable".to_string())
    );
    assert_eq!(std::fs::read(path).unwrap(), corrupt);
}

#[test]
fn corrupt_catalog_helper_reports_unavailable_instead_of_returning_empty() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    std::fs::write(&path, b"{not valid json").unwrap();

    assert_eq!(
        ModelCustomizationCatalog::read_from_path(&path).err(),
        Some("ollama-custom-store-unavailable".to_string())
    );
}

#[test]
fn unavailable_customization_store_recovers_after_a_valid_file_is_restored() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    std::fs::write(&path, b"{not valid json").unwrap();
    let store = ModelCustomizationStore::open(path.clone());
    let mut restored = ModelCustomizationCatalog::default();
    restored.mark_modelfile("legacy:latest").unwrap();
    restored.write_to_path(&path).unwrap();

    assert_eq!(
        store.kind("legacy:latest").unwrap(),
        Some(CustomizationKind::Modelfile)
    );
    store.mark_parameters("new:latest").unwrap();
    assert_eq!(
        store.kind("new:latest").unwrap(),
        Some(CustomizationKind::ParametersOnly)
    );
}

#[test]
fn customization_store_does_not_overwrite_corruption_that_happens_after_open() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    let store = ModelCustomizationStore::open(path.clone());
    store.mark_parameters("first:latest").unwrap();
    let corrupt = b"{corrupted while Beaver is running";
    std::fs::write(&path, corrupt).unwrap();

    assert_eq!(
        store.mark_parameters("second:latest"),
        Err("ollama-custom-store-unavailable".to_string())
    );
    assert_eq!(std::fs::read(path).unwrap(), corrupt);
}

#[test]
fn invalid_model_name_is_not_reported_as_customized() {
    let directory = tempfile::tempdir().unwrap();
    let store = ModelCustomizationStore::open(
        directory.path().join("ollama-custom-models.json"),
    );

    assert_eq!(customization_kind_from(&store, ""), None);
    assert_eq!(customization_kind_from(&store, "modèle:latest"), None);
    assert_eq!(customization_kind_from(&store, "bad..name"), None);
}

#[cfg(unix)]
#[test]
fn broken_store_symlink_is_unknown_and_never_replaced() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    symlink(directory.path().join("missing-target.json"), &path).unwrap();
    let store = ModelCustomizationStore::open(path.clone());

    assert_eq!(
        customization_kind_from(&store, "legacy:latest"),
        Some(CustomizationKind::Unknown)
    );
    assert!(store.mark_parameters("legacy:latest").is_err());
    assert!(std::fs::symlink_metadata(path)
        .unwrap()
        .file_type()
        .is_symlink());
}
