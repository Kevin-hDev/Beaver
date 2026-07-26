use crate::models::{ClgoConfig, MascotSettings};

#[test]
fn full_config_save_cannot_reset_dedicated_mascot_settings() {
    let current = ClgoConfig {
        mascot: MascotSettings {
            enabled: true,
            size_percent: 125,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut incoming = ClgoConfig::default();

    super::config::keep_current_mascot(&mut incoming, &current);

    assert!(incoming.mascot.enabled);
    assert_eq!(incoming.mascot.size_percent, 125);
}
