use super::*;

#[test]
fn protect_advanced_settings_keeps_existing_allowed_paths() {
    let mut current = ClgoConfig::default();
    current.advanced.allowed_paths = vec!["/trusted".to_string()];

    let incoming = AdvancedSettings {
        allowed_paths: vec!["/attacker".to_string()],
        ..Default::default()
    };

    let protected = protect_advanced_settings(incoming, &current);
    assert_eq!(protected.allowed_paths, vec!["/trusted"]);
}

#[test]
fn normalize_clears_start_hidden_when_autostart_is_disabled() {
    let settings = AdvancedSettings {
        autostart: false,
        start_hidden: true,
        ..Default::default()
    };

    let normalized = normalize_advanced_settings(settings);

    assert!(!normalized.autostart);
    assert!(!normalized.start_hidden);
}

#[test]
fn normalize_keeps_start_hidden_when_autostart_is_enabled() {
    let settings = AdvancedSettings {
        autostart: true,
        start_hidden: true,
        ..Default::default()
    };

    let normalized = normalize_advanced_settings(settings);

    assert!(normalized.autostart);
    assert!(normalized.start_hidden);
}

#[test]
fn normalize_clamps_compression_threshold() {
    let settings = AdvancedSettings {
        compression_threshold: 150,
        ..Default::default()
    };

    let normalized = normalize_advanced_settings(settings);

    assert_eq!(normalized.compression_threshold, 100);
}

#[test]
fn output_setting_rejects_a_missing_or_relative_directory() {
    for directory in ["../outside", "/definitely/missing/beaver-outputs"] {
        let mut settings = AdvancedSettings {
            session_outputs_directory: directory.to_string(),
            ..Default::default()
        };

        assert!(validate_outputs_directory(&mut settings).is_err());
    }
}
