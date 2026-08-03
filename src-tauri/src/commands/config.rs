use crate::models::{AdvancedSettings, ClgoConfig};
use crate::services::{autostart_migration, config as config_service};

#[tauri::command]
pub fn get_config() -> Result<ClgoConfig, String> {
    config_service::read_config()
}

#[tauri::command]
pub fn save_config(mut config: ClgoConfig) -> Result<(), String> {
    validate_outputs_directory(&mut config.advanced)?;
    config_service::update_config(move |current| {
        config.advanced = protect_advanced_settings(config.advanced, current);
        keep_current_mascot(&mut config, current);
        *current = config;
        Ok(())
    })
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
    let mut settings = settings;
    validate_outputs_directory(&mut settings)?;
    let settings = normalize_advanced_settings(settings);
    config_service::update_config(move |config| {
        let autostart_changed = settings.autostart != config.advanced.autostart;
        if autostart_changed {
            autostart_migration::synchronize_for_settings(&app, settings.autostart)?;
        }
        config.advanced = protect_advanced_settings(settings, config);
        Ok(())
    })
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

fn validate_outputs_directory(settings: &mut AdvancedSettings) -> Result<(), String> {
    settings.session_outputs_directory =
        crate::models::config::normalize_optional_directory(&settings.session_outputs_directory)
            .ok_or_else(|| "Dossier de sortie invalide.".to_string())?;
    Ok(())
}

#[cfg(test)]
#[path = "config_settings_tests.rs"]
mod tests;

const PATCH_BLOCKED_KEYS: &[&str] = &["allowed_paths"];

#[tauri::command]
pub fn patch_advanced_settings(
    app: tauri::AppHandle,
    patch: serde_json::Value,
) -> Result<(), String> {
    config_service::update_config(move |config| {
        let mut current = serde_json::to_value(&config.advanced).map_err(|e| {
            eprintln!("[config] serialize: {e}");
            "Erreur de configuration".to_string()
        })?;
        if let (Some(base), Some(updates)) = (current.as_object_mut(), patch.as_object()) {
            for (key, value) in updates {
                if !PATCH_BLOCKED_KEYS.contains(&key.as_str()) {
                    base.insert(key.clone(), value.clone());
                }
            }
        }
        let mut merged: AdvancedSettings = serde_json::from_value(current).map_err(|e| {
            eprintln!("[config] deserialize: {e}");
            "Erreur de configuration".to_string()
        })?;
        validate_outputs_directory(&mut merged)?;
        let merged = normalize_advanced_settings(merged);
        let autostart_requested = patch
            .as_object()
            .is_some_and(|updates| updates.contains_key("autostart"));
        let autostart_changed = merged.autostart != config.advanced.autostart;
        if autostart_requested || autostart_changed {
            autostart_migration::synchronize_for_settings(&app, merged.autostart)?;
        }
        config.advanced = merged;
        Ok(())
    })
}

#[tauri::command]
pub fn get_effective_context_length() -> u32 {
    crate::services::gpu_detect::compute_default_num_ctx()
}
