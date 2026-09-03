use super::ui_catalog::StoredCatalog;
use super::ui_types::UiCatalogSnapshot;
use serde::Serialize;
use std::collections::BTreeMap;

pub(super) fn validate(
    extensions: &BTreeMap<String, StoredCatalog>,
    revision: u64,
) -> Result<(), String> {
    let mut count = 0usize;
    let mut occupants = BTreeMap::<&str, usize>::new();
    for stored in extensions.values() {
        count = count
            .checked_add(stored.entries.len())
            .ok_or_else(unavailable)?;
        if count > super::ui_contract::MAX_GLOBAL_STANDARD_CONTRIBUTIONS {
            return Err(unavailable());
        }
        for entry in &stored.entries {
            if let Some(placement) = entry
                .contribution
                .get("placement")
                .and_then(serde_json::Value::as_str)
            {
                let value = occupants.entry(placement).or_default();
                *value = value.checked_add(1).ok_or_else(unavailable)?;
                if *value > super::ui_contract::MAX_OCCUPANTS_PER_PLACEMENT {
                    return Err(unavailable());
                }
            }
        }
    }
    if serialized_len(&snapshot(revision, extensions))? > super::ui_contract::MAX_GLOBAL_UI_BYTES {
        return Err(unavailable());
    }
    Ok(())
}

pub(super) fn snapshot(
    revision: u64,
    extensions: &BTreeMap<String, StoredCatalog>,
) -> UiCatalogSnapshot {
    UiCatalogSnapshot {
        revision,
        contributions: extensions
            .values()
            .flat_map(|stored| stored.entries.iter().cloned())
            .collect(),
    }
}

fn serialized_len(value: &impl Serialize) -> Result<usize, String> {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .map_err(|_| unavailable())
}

fn unavailable() -> String {
    super::error_codes::OPERATION_FAILED.to_string()
}
