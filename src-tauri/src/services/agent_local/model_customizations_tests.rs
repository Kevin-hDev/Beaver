use super::model_customizations::{
    CustomizationKind, ModelCustomizationCatalog, ModelCustomizationStore,
};
use std::sync::{Arc, Barrier};

#[test]
fn legacy_model_names_are_migrated_as_unknown_customizations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ollama-custom-models.json");
    std::fs::write(&path, r#"{"models":["gemma4:e2b"]}"#).unwrap();

    let catalog = ModelCustomizationCatalog::read_from_path(&path);

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
    let mut catalog = ModelCustomizationCatalog::read_from_path(&path);
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
    let loaded = ModelCustomizationCatalog::read_from_path(&path);

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

    let persisted = ModelCustomizationCatalog::read_from_path(&path);
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
    let loaded = ModelCustomizationCatalog::read_from_path(&path);
    assert_eq!(
        loaded.kind(&format!("model-0-{suffix}")),
        Some(CustomizationKind::ParametersOnly)
    );
    assert_eq!(
        loaded.kind(&format!("model-511-{suffix}")),
        Some(CustomizationKind::ParametersOnly)
    );
}
