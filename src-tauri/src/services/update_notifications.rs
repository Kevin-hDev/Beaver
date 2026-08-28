use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: u8 = 1;
const SUBJECT_MAX_CHARS: usize = 256;
const VERSION_MAX_CHARS: usize = 128;
pub const MAX_DISMISSED_UPDATES: usize = 128;

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DismissedUpdateKind {
    App,
    OllamaBinary,
    OllamaModel,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct DismissedUpdate {
    pub kind: DismissedUpdateKind,
    pub subject: String,
    pub version: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct DismissedUpdateStore {
    schema_version: u8,
    dismissed: Vec<DismissedUpdate>,
}

pub fn store_path() -> PathBuf {
    crate::services::paths::data_dir().join("update-notifications.json")
}

pub fn read() -> Result<Vec<DismissedUpdate>, String> {
    read_from_path(&store_path())
}

pub fn dismiss(update: DismissedUpdate) -> Result<Vec<DismissedUpdate>, String> {
    dismiss_at_path(&store_path(), update)
}

pub(crate) fn read_from_path(path: &Path) -> Result<Vec<DismissedUpdate>, String> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(store_error()),
    };
    let store: DismissedUpdateStore = match serde_json::from_str(&content) {
        Ok(store) => store,
        Err(_) => {
            // A broken preference must reveal updates again, never hide them indefinitely.
            log::warn!("Ignoring invalid update notification preferences");
            return Ok(Vec::new());
        }
    };
    if store.schema_version != SCHEMA_VERSION
        || store.dismissed.len() > MAX_DISMISSED_UPDATES
        || store.dismissed.iter().any(|item| validate(item).is_err())
    {
        log::warn!("Ignoring unsupported update notification preferences");
        return Ok(Vec::new());
    }
    Ok(store.dismissed)
}

pub(crate) fn dismiss_at_path(
    path: &Path,
    update: DismissedUpdate,
) -> Result<Vec<DismissedUpdate>, String> {
    validate(&update)?;
    let mut dismissed = read_from_path(path)?;
    dismissed.retain(|item| item.kind != update.kind || item.subject != update.subject);
    dismissed.push(update);
    if dismissed.len() > MAX_DISMISSED_UPDATES {
        let overflow = dismissed.len() - MAX_DISMISSED_UPDATES;
        dismissed.drain(..overflow);
    }
    let store = DismissedUpdateStore {
        schema_version: SCHEMA_VERSION,
        dismissed: dismissed.clone(),
    };
    let content = serde_json::to_vec_pretty(&store).map_err(|_| store_error())?;
    crate::services::private_store::atomic_write(path, &content).map_err(|_| store_error())?;
    Ok(dismissed)
}

fn validate(update: &DismissedUpdate) -> Result<(), String> {
    if !valid_token(&update.subject, SUBJECT_MAX_CHARS)
        || !valid_token(&update.version, VERSION_MAX_CHARS)
        || update.subject.contains("..")
    {
        return Err(store_error());
    }
    Ok(())
}

fn valid_token(value: &str, max_chars: usize) -> bool {
    !value.is_empty()
        && value.chars().count() <= max_chars
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '-' | '_' | '.' | ':' | '/' | '@' | '+')
        })
}

fn store_error() -> String {
    "update-notifications-store-error".to_string()
}
