use super::validation;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

static POLICY_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
const MAX_POLICY_BYTES: usize = 4 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForecastSelectionMode {
    Manual,
    Auto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastSelectionPolicy {
    pub mode: ForecastSelectionMode,
    pub manual_model_id: Option<String>,
    #[serde(default)]
    pub allow_cloud_in_auto: bool,
}

#[derive(Deserialize)]
struct LegacySelectedModel {
    model: String,
}

impl Default for ForecastSelectionPolicy {
    fn default() -> Self {
        Self {
            mode: ForecastSelectionMode::Manual,
            manual_model_id: None,
            allow_cloud_in_auto: false,
        }
    }
}

pub fn get() -> Result<ForecastSelectionPolicy, String> {
    let _guard = POLICY_LOCK.lock().map_err(|_| storage_error())?;
    load_or_migrate(&policy_path(), &legacy_path())
}

pub fn set_mode(mode: ForecastSelectionMode) -> Result<ForecastSelectionPolicy, String> {
    let _guard = POLICY_LOCK.lock().map_err(|_| storage_error())?;
    let mut policy = load_or_migrate(&policy_path(), &legacy_path())?;
    policy.mode = mode;
    store(&policy)?;
    Ok(policy)
}

pub fn select_manual_model(model: &str) -> Result<ForecastSelectionPolicy, String> {
    validation::validate_runnable_model_id(model)?;
    let _guard = POLICY_LOCK.lock().map_err(|_| storage_error())?;
    let mut policy = load_or_migrate(&policy_path(), &legacy_path())?;
    policy.mode = ForecastSelectionMode::Manual;
    policy.manual_model_id = Some(model.to_string());
    store(&policy)?;
    Ok(policy)
}

pub fn set_cloud_allowed(allowed: bool) -> Result<ForecastSelectionPolicy, String> {
    let _guard = POLICY_LOCK.lock().map_err(|_| storage_error())?;
    let mut policy = load_or_migrate(&policy_path(), &legacy_path())?;
    policy.allow_cloud_in_auto = allowed;
    store(&policy)?;
    Ok(policy)
}

fn store(policy: &ForecastSelectionPolicy) -> Result<(), String> {
    validate(policy)?;
    write_policy(&policy_path(), policy)
}

fn policy_path() -> PathBuf {
    crate::services::paths::data_dir().join("forecast-selection-policy.json")
}

fn legacy_path() -> PathBuf {
    crate::services::paths::data_dir().join("forecast-selected-model.json")
}

fn load_or_migrate(policy: &Path, legacy: &Path) -> Result<ForecastSelectionPolicy, String> {
    match read_bounded(policy) {
        Ok(bytes) => parse_policy(&bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => migrate(policy, legacy),
        Err(_) => Err(storage_error()),
    }
}

fn migrate(policy_path: &Path, legacy_path: &Path) -> Result<ForecastSelectionPolicy, String> {
    let mut policy = ForecastSelectionPolicy::default();
    match read_bounded(legacy_path) {
        Ok(bytes) => {
            let legacy: LegacySelectedModel =
                serde_json::from_slice(&bytes).map_err(|_| storage_error())?;
            validation::validate_runnable_model_id(&legacy.model).map_err(|_| storage_error())?;
            policy.manual_model_id = Some(legacy.model);
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => return Err(storage_error()),
    }
    write_policy(policy_path, &policy)?;
    Ok(policy)
}

fn parse_policy(bytes: &[u8]) -> Result<ForecastSelectionPolicy, String> {
    let policy: ForecastSelectionPolicy =
        serde_json::from_slice(bytes).map_err(|_| storage_error())?;
    validate(&policy).map_err(|_| storage_error())?;
    Ok(policy)
}

fn read_bounded(path: &Path) -> std::io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut bytes = Vec::with_capacity(MAX_POLICY_BYTES.min(512));
    file.take((MAX_POLICY_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_POLICY_BYTES {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "forecast policy too large",
        ));
    }
    Ok(bytes)
}

fn validate(policy: &ForecastSelectionPolicy) -> Result<(), String> {
    if let Some(model) = policy.manual_model_id.as_deref() {
        validation::validate_runnable_model_id(model)?;
    }
    Ok(())
}

fn write_policy(path: &Path, policy: &ForecastSelectionPolicy) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(policy).map_err(|_| storage_error())?;
    crate::services::private_store::atomic_write(path, &bytes).map_err(|_| storage_error())
}

fn storage_error() -> String {
    "Politique Forecast indisponible".to_string()
}

#[cfg(test)]
#[path = "selection_policy_tests.rs"]
mod tests;
