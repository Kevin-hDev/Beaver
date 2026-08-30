use super::*;

#[test]
fn legacy_compression_settings_are_not_a_serialized_authority() {
    let settings: AdvancedSettings = serde_json::from_value(serde_json::json!({
        "compression_enabled": false,
        "compression_threshold": 85
    }))
    .expect("legacy settings");
    assert_eq!(settings.legacy_compression_enabled, Some(false));
    assert_eq!(settings.legacy_compression_threshold, Some(85));

    let serialized = serde_json::to_value(settings).expect("serialized settings");
    assert!(serialized.get("compression_enabled").is_none());
    assert!(serialized.get("compression_threshold").is_none());
}

#[test]
fn ollama_setup_is_not_skipped_by_default() {
    let settings = AdvancedSettings::default();
    assert!(!settings.ollama_setup_skipped);
}

#[test]
fn onboarding_is_not_completed_by_default() {
    let settings = AdvancedSettings::default();
    assert!(!settings.onboarding_completed);
}

#[test]
fn relative_outputs_directory_is_cleared() {
    let settings = AdvancedSettings {
        session_outputs_directory: "../outside".to_string(),
        ..Default::default()
    }
    .normalized();

    assert!(settings.session_outputs_directory.is_empty());
}

#[test]
fn missing_absolute_outputs_directory_is_preserved() {
    let path = std::env::temp_dir().join(format!("beaver-offline-output-{}", uuid::Uuid::new_v4()));
    let settings = AdvancedSettings {
        session_outputs_directory: path.to_string_lossy().to_string(),
        ..Default::default()
    }
    .normalized();

    assert_eq!(settings.session_outputs_directory, path.to_string_lossy());
    assert!(existing_optional_directory(&settings.session_outputs_directory).is_none());
}

#[test]
fn legacy_empty_allowed_paths_are_migrated_to_the_safe_default() {
    let settings = AdvancedSettings {
        allowed_paths: Vec::new(),
        ..Default::default()
    }
    .normalized();

    assert_eq!(settings.allowed_paths, default_allowed_paths());
}
