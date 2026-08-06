use super::model_customizations::{CustomizationKind, ModelCustomizationCatalog};

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
