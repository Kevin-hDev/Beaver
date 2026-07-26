use crate::models::{AdvancedSettings, ClgoConfig};
use crate::services::{autostart_migration, config as config_service};

#[tauri::command]
pub fn get_config() -> Result<ClgoConfig, String> {
    config_service::read_config()
}

#[tauri::command]
pub fn save_config(mut config: ClgoConfig) -> Result<(), String> {
    let current = config_service::read_config()?;
    config.advanced = protect_advanced_settings(config.advanced, &current);
    keep_current_mascot(&mut config, &current);
    config_service::write_config(&config)
}

pub(crate) fn keep_current_mascot(config: &mut ClgoConfig, current: &ClgoConfig) {
    config.mascot = current.mascot.clone();
}

#[tauri::command]
pub fn get_advanced_settings() -> Result<AdvancedSettings, String> {
    let config = config_service::read_config()?;
    Ok(config.advanced)
}

#[tauri::command]
pub fn set_advanced_settings(
    app: tauri::AppHandle,
    settings: AdvancedSettings,
) -> Result<(), String> {
    let settings = normalize_advanced_settings(settings);
    let mut config = config_service::read_config()?;
    let autostart_changed = settings.autostart != config.advanced.autostart;
    if autostart_changed {
        autostart_migration::synchronize_for_settings(&app, settings.autostart)?;
    }
    config.advanced = protect_advanced_settings(settings, &config);
    config_service::write_config(&config)
}

fn protect_advanced_settings(
    mut settings: AdvancedSettings,
    current: &ClgoConfig,
) -> AdvancedSettings {
    settings.allowed_paths = current.advanced.allowed_paths.clone();
    settings
}

fn normalize_advanced_settings(mut settings: AdvancedSettings) -> AdvancedSettings {
    settings = settings.normalized();
    if !settings.autostart {
        settings.start_hidden = false;
    }
    settings
}

#[cfg(test)]
mod tests {
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
}

const PATCH_BLOCKED_KEYS: &[&str] = &["allowed_paths"];

#[tauri::command]
pub fn patch_advanced_settings(
    app: tauri::AppHandle,
    patch: serde_json::Value,
) -> Result<(), String> {
    let mut config = config_service::read_config()?;
    let mut current = serde_json::to_value(&config.advanced).map_err(|e| {
        eprintln!("[config] serialize: {e}");
        "Erreur de configuration".to_string()
    })?;

    if let (Some(base), Some(updates)) = (current.as_object_mut(), patch.as_object()) {
        for (k, v) in updates {
            if PATCH_BLOCKED_KEYS.contains(&k.as_str()) {
                continue;
            }
            base.insert(k.clone(), v.clone());
        }
    }

    let merged: AdvancedSettings = serde_json::from_value(current).map_err(|e| {
        eprintln!("[config] deserialize: {e}");
        "Erreur de configuration".to_string()
    })?;
    let merged = normalize_advanced_settings(merged);
    let autostart_requested = patch
        .as_object()
        .map(|updates| updates.contains_key("autostart"))
        .unwrap_or(false);
    let autostart_changed = merged.autostart != config.advanced.autostart;

    if autostart_requested || autostart_changed {
        autostart_migration::synchronize_for_settings(&app, merged.autostart)?;
    }

    config.advanced = merged;
    config_service::write_config(&config)
}

#[tauri::command]
pub fn get_effective_context_length() -> u32 {
    crate::services::gpu_detect::compute_default_num_ctx()
}
