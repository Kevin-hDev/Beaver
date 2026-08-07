use crate::models::{
    AdvancedSettings, ClgoConfig, GatewayConfig, HeartbeatConfig, MascotSettings, ScheduledWakeup,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, RwLock};

static SESSION_OUTPUTS_DIRECTORY: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();
static CONFIG_UPDATE_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn config_path() -> PathBuf {
    crate::services::paths::data_dir().join("config.json")
}

/// Lecture tolérante du config :
/// - fichier absent → config par défaut (vide)
/// - JSON corrompu → config par défaut + log
/// - wakeups au format obsolète (CL-GO legacy) → ignorés un par un + log
pub fn read_config() -> Result<ClgoConfig, String> {
    read_config_unlocked()
}

fn read_config_unlocked() -> Result<ClgoConfig, String> {
    let mut config = read_config_from_path(&config_path(), &crate::services::paths::data_dir())?;
    crate::services::agent_local::directory_access::apply_cached_policy(&mut config);
    cache_session_outputs_directory(&config.advanced.session_outputs_directory);
    Ok(config)
}

/// Variante testable : lit le config depuis `path` et écrit la sentinelle de
/// corruption dans `data_dir`. La logique de parsing tolérant vit ici.
pub(crate) fn read_config_from_path(path: &Path, data_dir: &Path) -> Result<ClgoConfig, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(ClgoConfig::default()),
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            ::log::warn!("[config] JSON invalide ({}), reset à zéro", e);
            let sentinel = data_dir.join(".config-corrupted");
            let _ = fs::write(&sentinel, format!("{}", e));
            return Ok(ClgoConfig::default());
        }
    };

    let mut config = ClgoConfig::default();
    let Some(obj) = value.as_object() else {
        return Ok(config);
    };

    if let Some(hb) = obj.get("heartbeat") {
        config.heartbeat =
            serde_json::from_value::<HeartbeatConfig>(hb.clone()).unwrap_or_default();
    }

    if let Some(adv) = obj.get("advanced") {
        config.advanced = serde_json::from_value::<AdvancedSettings>(adv.clone())
            .unwrap_or_default()
            .normalized();
    }

    if let Some(gw) = obj.get("gateway") {
        config.gateway = serde_json::from_value::<GatewayConfig>(gw.clone()).unwrap_or_default();
    }

    if let Some(mascot) = obj.get("mascot") {
        config.mascot = serde_json::from_value::<MascotSettings>(mascot.clone())
            .unwrap_or_default()
            .normalized();
    }

    if let Some(arr) = obj.get("scheduled_wakeups").and_then(|v| v.as_array()) {
        let mut dropped = 0u32;
        for item in arr {
            match serde_json::from_value::<ScheduledWakeup>(item.clone()) {
                Ok(w) => config.scheduled_wakeups.push(w),
                Err(_) => dropped += 1,
            }
        }
        if dropped > 0 {
            ::log::warn!(
                "[config] {} wakeup(s) au format obsolète ignoré(s)",
                dropped
            );
        }
    }

    Ok(config)
}

pub fn update_config<T>(
    update: impl FnOnce(&mut ClgoConfig) -> Result<T, String>,
) -> Result<T, String> {
    let _guard = CONFIG_UPDATE_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut config = read_config_unlocked()?;
    let result = update(&mut config)?;
    write_config_unlocked(&config)?;
    Ok(result)
}

fn write_config_unlocked(config: &ClgoConfig) -> Result<(), String> {
    write_config_to_path(&config_path(), config)?;
    cache_session_outputs_directory(&config.advanced.session_outputs_directory);
    Ok(())
}

pub(crate) fn read_allowed_paths_strict() -> Result<Vec<String>, String> {
    read_allowed_paths_strict_from_path(&config_path())
}

pub(crate) fn read_allowed_paths_strict_from_path(path: &Path) -> Result<Vec<String>, String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(crate::models::config::default_allowed_paths());
        }
        Err(_) => return Err("Politique d’accès indisponible.".to_string()),
    };
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|_| "Politique d’accès indisponible.".to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "Politique d’accès indisponible.".to_string())?;
    let Some(advanced) = object.get("advanced") else {
        return Ok(crate::models::config::default_allowed_paths());
    };
    let settings = serde_json::from_value::<AdvancedSettings>(advanced.clone())
        .map_err(|_| "Politique d’accès indisponible.".to_string())?
        .normalized();
    Ok(settings.allowed_paths)
}

pub fn session_outputs_directory() -> Option<PathBuf> {
    if let Some(cache) = SESSION_OUTPUTS_DIRECTORY.get() {
        return read_cached_directory(cache);
    }
    read_config()
        .ok()
        .and_then(|config| {
            crate::models::config::normalize_optional_directory(
                &config.advanced.session_outputs_directory,
            )
        })
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn cache_session_outputs_directory(value: &str) {
    let value = crate::models::config::normalize_optional_directory(value)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let cache = SESSION_OUTPUTS_DIRECTORY.get_or_init(|| RwLock::new(None));
    match cache.write() {
        Ok(mut cached) => *cached = value,
        Err(poisoned) => *poisoned.into_inner() = value,
    }
}

fn read_cached_directory(cache: &RwLock<Option<PathBuf>>) -> Option<PathBuf> {
    match cache.read() {
        Ok(cached) => cached.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Variante testable : écrit atomiquement (tmp + rename) le config vers `path`.
pub(crate) fn write_config_to_path(path: &Path, config: &ClgoConfig) -> Result<(), String> {
    let content = serde_json::to_string_pretty(config)
        .map_err(|_| "écriture de la configuration impossible".to_string())?;
    crate::services::private_store::atomic_write(path, content.as_bytes())
        .map_err(|_| "écriture de la configuration impossible".to_string())
}

#[cfg(test)]
#[path = "config_resilience_tests.rs"]
mod resilience_tests;
