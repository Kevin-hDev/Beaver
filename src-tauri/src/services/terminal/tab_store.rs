use super::limits::{
    MAX_GROUPS, MAX_GROUP_KEY_BYTES, MAX_LABEL_BYTES, MAX_TABS_PER_GROUP, MAX_TOTAL_TABS,
};
use super::tab_store_recovery::{
    recover_invalid, recover_oversized, resume_oversized_recovery, RECOVERED,
};
use crate::services::private_store::{
    self, read_bounded_regular_classified_async, BoundedFile, BoundedReadFailure,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const TERMINAL_TABS_VERSION: u8 = 1;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const INVALID: &str = "terminal-tabs-invalid";
const UNAVAILABLE: &str = "terminal-tabs-unavailable";

static TERMINAL_TABS_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSavedTab {
    pub label: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct TerminalTabsDocument {
    pub version: u8,
    pub groups: BTreeMap<String, Vec<TerminalSavedTab>>,
}

#[derive(serde::Deserialize)]
struct LegacyTab {
    label: String,
    #[serde(rename = "cwd")]
    _cwd: Option<String>,
}

impl TerminalTabsDocument {
    pub fn empty() -> Self {
        Self {
            version: TERMINAL_TABS_VERSION,
            groups: BTreeMap::new(),
        }
    }
}

pub async fn load() -> Result<TerminalTabsDocument, String> {
    let _guard = TERMINAL_TABS_LOCK.lock().await;
    load_from(path()).await
}

pub(super) async fn load_from(path: PathBuf) -> Result<TerminalTabsDocument, String> {
    let pending = path.with_extension("json.recovery-pending");
    if available(tokio::fs::try_exists(&pending).await)?
        && resume_oversized_recovery(&path, &pending).await?
    {
        return Err(RECOVERED.to_string());
    }
    match read_bounded_regular_classified_async(path.clone(), MAX_FILE_BYTES).await {
        Ok(BoundedFile::Missing) => Ok(TerminalTabsDocument::empty()),
        Ok(BoundedFile::Content(content)) => match parse_document(&content) {
            Ok(document) => Ok(document),
            Err(_) => recover_invalid(path, content).await,
        },
        Err(BoundedReadFailure::TooLarge) => recover_oversized(path).await,
        Err(BoundedReadFailure::Unavailable) => Err(unavailable()),
    }
}

pub(super) fn available<T, E>(result: Result<T, E>) -> Result<T, String> {
    result.map_err(|_| unavailable())
}

pub async fn save(document: TerminalTabsDocument) -> Result<(), String> {
    save_with(document, private_store::atomic_write_async).await
}

pub(crate) fn serialize_document(document: &TerminalTabsDocument) -> Result<Vec<u8>, String> {
    validate_document(document)?;
    let bytes = serde_json::to_vec(document).map_err(|_| invalid())?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(invalid());
    }
    Ok(bytes)
}

pub(super) async fn save_with<Writer, Future>(
    document: TerminalTabsDocument,
    writer: Writer,
) -> Result<(), String>
where
    Writer: FnOnce(PathBuf, Vec<u8>) -> Future,
    Future: std::future::Future<Output = Result<(), String>>,
{
    let _guard = TERMINAL_TABS_LOCK.lock().await;
    let bytes = serialize_document(&document)?;
    writer(path(), bytes).await.map_err(|_| unavailable())
}

pub(super) fn parse_document(bytes: &[u8]) -> Result<TerminalTabsDocument, String> {
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(invalid());
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    if value
        .as_object()
        .is_some_and(|object| object.contains_key("version") || object.contains_key("groups"))
    {
        let document = serde_json::from_value(value).map_err(|_| invalid())?;
        validate_document(&document)?;
        return Ok(document);
    }

    let groups = match value {
        Value::Object(_) => {
            let legacy: BTreeMap<String, Vec<LegacyTab>> =
                serde_json::from_value(value).map_err(|_| invalid())?;
            legacy
                .into_iter()
                .map(|(group, tabs)| (group, migrate_tabs(tabs)))
                .collect()
        }
        Value::Array(_) => {
            let legacy: Vec<LegacyTab> = serde_json::from_value(value).map_err(|_| invalid())?;
            BTreeMap::from([("__default__".to_string(), migrate_tabs(legacy))])
        }
        _ => return Err(invalid()),
    };
    let document = TerminalTabsDocument {
        version: TERMINAL_TABS_VERSION,
        groups,
    };
    validate_document(&document)?;
    Ok(document)
}

pub(super) fn validate_document(document: &TerminalTabsDocument) -> Result<(), String> {
    if document.version != TERMINAL_TABS_VERSION || document.groups.len() > MAX_GROUPS {
        return Err(invalid());
    }
    let mut total_tabs = 0_usize;
    for (group, tabs) in &document.groups {
        if !bounded_text(group, MAX_GROUP_KEY_BYTES) || tabs.len() > MAX_TABS_PER_GROUP {
            return Err(invalid());
        }
        total_tabs = total_tabs.checked_add(tabs.len()).ok_or_else(invalid)?;
        if total_tabs > MAX_TOTAL_TABS {
            return Err(invalid());
        }
        if tabs
            .iter()
            .any(|tab| !bounded_text(&tab.label, MAX_LABEL_BYTES))
        {
            return Err(invalid());
        }
    }
    Ok(())
}

fn migrate_tabs(tabs: Vec<LegacyTab>) -> Vec<TerminalSavedTab> {
    tabs.into_iter()
        .map(|tab| {
            let LegacyTab { label, _cwd: _ } = tab;
            TerminalSavedTab { label }
        })
        .collect()
}

fn bounded_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && !value
            .as_bytes()
            .iter()
            .any(|byte| matches!(byte, 0 | b'\r' | b'\n'))
}

fn path() -> PathBuf {
    crate::services::paths::data_dir().join("terminal-tabs.json")
}

fn invalid() -> String {
    INVALID.to_string()
}

pub(super) fn unavailable() -> String {
    UNAVAILABLE.to_string()
}
