use super::store_wire::{decode_file, AutomationFile};
use crate::models::AutomationDefinition;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const AUTOMATIONS_SCHEMA_VERSION: u32 = 1;
const MAX_AUTOMATIONS: usize = 64;
const MAX_STORE_BYTES: u64 = 2 * 1024 * 1024;

pub async fn read_all() -> Result<Vec<AutomationDefinition>, String> {
    read_all_at(&crate::services::paths::data_dir()).await
}

pub(crate) async fn read_all_at(root: &Path) -> Result<Vec<AutomationDefinition>, String> {
    let _guard = super::store_lock().await;
    read_all_unlocked_at(root).await
}

#[cfg(test)]
pub async fn mutate<R>(
    change: impl FnOnce(&mut Vec<AutomationDefinition>) -> Result<R, String>,
) -> Result<R, String> {
    mutate_at(&crate::services::paths::data_dir(), change).await
}

#[cfg(test)]
pub(crate) async fn mutate_at<R>(
    root: &Path,
    change: impl FnOnce(&mut Vec<AutomationDefinition>) -> Result<R, String>,
) -> Result<R, String> {
    let _guard = super::store_lock().await;
    let mut automations = read_all_unlocked_at(root).await?;
    let result = change(&mut automations)?;
    write_definitions_unlocked_at(root, automations).await?;
    Ok(result)
}

#[cfg(test)]
pub(crate) async fn write_file_at(root: &Path, file: &AutomationFile) -> Result<(), String> {
    let _guard = super::store_lock().await;
    write_definitions_unlocked_at(root, decode_file(file)?).await
}

pub(crate) async fn read_all_unlocked_at(root: &Path) -> Result<Vec<AutomationDefinition>, String> {
    let Some(file) = read_file_unlocked_at(root).await? else {
        return Ok(Vec::new());
    };
    let definitions = decode_file(&file)?;
    validate(&definitions)?;
    Ok(definitions)
}

pub(crate) async fn write_definitions_unlocked_at(
    root: &Path,
    automations: Vec<AutomationDefinition>,
) -> Result<(), String> {
    validate(&automations)?;
    let bytes = serde_json::to_vec_pretty(&AutomationFile::from_definitions(automations))
        .map_err(|_| store_error())?;
    crate::services::private_store::atomic_write_async(path(root), bytes)
        .await
        .map_err(|_| store_error())
}

async fn read_file_unlocked_at(root: &Path) -> Result<Option<AutomationFile>, String> {
    match crate::services::private_store::read_bounded_regular_async(path(root), MAX_STORE_BYTES)
        .await?
    {
        crate::services::private_store::BoundedFile::Missing => Ok(None),
        crate::services::private_store::BoundedFile::Content(bytes) => {
            serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|_| store_error())
        }
    }
}

fn validate(items: &[AutomationDefinition]) -> Result<(), String> {
    let unique = items.iter().map(|item| item.id).collect::<HashSet<_>>();
    if items.len() > MAX_AUTOMATIONS || unique.len() != items.len() {
        return Err(store_error());
    }
    Ok(())
}

fn path(root: &Path) -> PathBuf {
    root.join("automations.json")
}

pub(super) fn store_error() -> String {
    "AUTOMATION_STORE_UNAVAILABLE".into()
}
