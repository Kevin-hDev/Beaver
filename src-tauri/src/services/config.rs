use crate::models::{
    AdvancedSettings, ClgoConfig, GatewayConfig, HeartbeatConfig, MascotSettings, ScheduledWakeup,
};
use std::fs;
use std::path::{Path, PathBuf};

fn config_path() -> PathBuf {
    crate::services::paths::data_dir().join("config.json")
}

/// Lecture tolérante du config :
/// - fichier absent → config par défaut (vide)
/// - JSON corrompu → config par défaut + log
/// - wakeups au format obsolète (CL-GO legacy) → ignorés un par un + log
pub fn read_config() -> Result<ClgoConfig, String> {
    read_config_from_path(&config_path(), &crate::services::paths::data_dir())
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
            eprintln!("[config] JSON invalide ({}), reset à zéro", e);
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
            eprintln!(
                "[config] {} wakeup(s) au format obsolète ignoré(s)",
                dropped
            );
        }
    }

    Ok(config)
}

pub fn write_config(config: &ClgoConfig) -> Result<(), String> {
    write_config_to_path(&config_path(), config)
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
