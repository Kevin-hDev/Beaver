use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

const MAX_MODELS: usize = 100;
const MAX_PARAMS_PER_MODEL: usize = 64;
const MAX_CONFIG_BYTES: u64 = 256 * 1024;

pub type StoredConfigs = BTreeMap<String, Map<String, Value>>;

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("forecast-model-configs.json")
}

pub fn read_all() -> Result<StoredConfigs, String> {
    let target = path();
    let metadata = match std::fs::metadata(&target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(_) => return Err("Configuration Forecast inaccessible".into()),
    };
    if metadata.len() > MAX_CONFIG_BYTES {
        return Err("Configuration Forecast invalide".into());
    }
    let content = match std::fs::read_to_string(target) {
        Ok(content) => content,
        Err(_) => return Err("Configuration Forecast inaccessible".into()),
    };
    parse_configs(&content)
}

pub fn read_model(model_id: &str) -> Result<Map<String, Value>, String> {
    Ok(read_all()?.remove(model_id).unwrap_or_default())
}

pub fn write_model(model_id: &str, values: Map<String, Value>) -> Result<(), String> {
    let mut all = read_all()?;
    if values.is_empty() {
        all.remove(model_id);
    } else {
        if !all.contains_key(model_id) && all.len() >= MAX_MODELS {
            return Err("Configuration Forecast trop volumineuse".into());
        }
        if values.len() > MAX_PARAMS_PER_MODEL {
            return Err("Configuration Forecast invalide".into());
        }
        all.insert(model_id.to_string(), values);
    }
    write_all(&all)
}

fn parse_configs(content: &str) -> Result<StoredConfigs, String> {
    let configs: StoredConfigs =
        serde_json::from_str(content).map_err(|_| "Configuration Forecast invalide")?;
    let invalid = configs.len() > MAX_MODELS
        || configs.iter().any(|(model_id, values)| {
            crate::services::forecast::validation::validate_model_id_format(model_id).is_err()
                || values.len() > MAX_PARAMS_PER_MODEL
        });
    if invalid {
        return Err("Configuration Forecast invalide".into());
    }
    Ok(configs)
}

fn write_all(all: &StoredConfigs) -> Result<(), String> {
    let target = path();
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| "Impossible d'enregistrer la configuration".to_string())?;
    }
    let body = serde_json::to_string_pretty(all)
        .map_err(|_| "Impossible d'enregistrer la configuration".to_string())?;
    let tmp = target.with_extension("tmp");
    std::fs::write(&tmp, body)
        .map_err(|_| "Impossible d'enregistrer la configuration".to_string())?;
    std::fs::rename(&tmp, &target)
        .map_err(|_| "Impossible d'enregistrer la configuration".to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_configs;

    #[test]
    fn corrupted_or_unsafe_configs_fail_closed() {
        assert!(parse_configs("not-json").is_err());
        assert!(parse_configs(r#"{"../model": {}}"#).is_err());
    }
}
