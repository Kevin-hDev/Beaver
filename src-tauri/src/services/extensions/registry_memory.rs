use super::types::ExtensionRecord;
use std::sync::{LazyLock, RwLock};

#[derive(Clone, Default)]
pub(super) struct RegistryMemory {
    pub(super) records: Vec<ExtensionRecord>,
    pub(super) recovery_snapshot: Option<Vec<String>>,
}

static STATE: LazyLock<RwLock<RegistryMemory>> =
    LazyLock::new(|| RwLock::new(RegistryMemory::default()));

pub(super) fn snapshot() -> Result<RegistryMemory, String> {
    STATE
        .read()
        .map(|state| state.clone())
        .map_err(|_| unavailable())
}

pub(super) fn records() -> Result<Vec<ExtensionRecord>, String> {
    snapshot().map(|state| state.records)
}

pub(super) fn with_records<T>(read: impl FnOnce(&[ExtensionRecord]) -> T) -> Result<T, String> {
    STATE
        .read()
        .map(|state| read(&state.records))
        .map_err(|_| unavailable())
}

pub(super) fn replace(
    records: Vec<ExtensionRecord>,
    recovery_snapshot: Option<Vec<String>>,
) -> Result<(), String> {
    let mut state = STATE.write().map_err(|_| unavailable())?;
    super::registry_index::rebuild(&records)?;
    *state = RegistryMemory {
        records,
        recovery_snapshot,
    };
    Ok(())
}

fn unavailable() -> String {
    "Registre d'extensions indisponible.".to_string()
}
