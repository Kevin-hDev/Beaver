use super::*;

#[test]
fn default_compression_settings() {
    let settings = AdvancedSettings::default();
    assert!(settings.compression_enabled);
    assert_eq!(settings.compression_threshold, 85);
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
fn compression_threshold_bounds() {
    let mut settings = AdvancedSettings {
        compression_threshold: 0,
        ..Default::default()
    };
    assert_eq!(settings.compression_threshold, 0);
    settings.compression_threshold = 100;
    assert_eq!(settings.compression_threshold, 100);
}

#[test]
fn compression_threshold_is_clamped() {
    let settings = AdvancedSettings {
        compression_threshold: 150,
        ..Default::default()
    }
    .normalized();
    assert_eq!(settings.compression_threshold, 100);
}

#[test]
fn invalid_outputs_directory_is_cleared() {
    let settings = AdvancedSettings {
        session_outputs_directory: "../outside".to_string(),
        ..Default::default()
    }
    .normalized();

    assert!(settings.session_outputs_directory.is_empty());
}

#[test]
fn outputs_directory_is_canonicalized() {
    let root = tempfile::tempdir().expect("tempdir");
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).expect("nested");

    let normalized =
        normalize_optional_directory(&nested.join(".").to_string_lossy()).expect("valid directory");

    assert_eq!(
        normalized,
        nested.canonicalize().expect("canonical").to_string_lossy()
    );
}
