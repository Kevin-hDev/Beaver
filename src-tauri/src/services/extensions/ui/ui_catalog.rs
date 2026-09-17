use super::host_identity::HostIdentity;
use super::ui_types::{UiCatalogEntry, UiCatalogSnapshot};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::RwLock;
use tauri::Emitter;

pub(super) const CHANGED_EVENT: &str = "extensions-ui-catalog-changed";

#[derive(Clone, PartialEq)]
pub(super) struct StoredCatalog {
    pub(super) identity: HostIdentity,
    pub(super) generation: u64,
    pub(super) catalog_revision: u64,
    pub(super) entries: Vec<UiCatalogEntry>,
}

#[derive(Clone, Default)]
pub(super) struct CatalogState {
    pub(super) revision: u64,
    pub(super) extensions: BTreeMap<String, StoredCatalog>,
    pub(super) retired: BTreeMap<HostIdentity, u64>,
    pub(super) retired_order: VecDeque<(HostIdentity, u64)>,
}

pub(super) struct UiCatalogUpdate {
    pub identity: HostIdentity,
    pub generation: u64,
    pub extension_id: String,
    pub entries: Vec<UiCatalogEntry>,
}

pub(super) struct UiCatalogApply {
    pub revision: u64,
    pub rejected_extensions: BTreeSet<String>,
}

#[derive(Clone)]
pub(super) struct UiActionRoute {
    pub identity: HostIdentity,
    pub generation: u64,
    pub catalog_revision: u64,
}

pub(super) struct UiCatalog {
    pub(super) state: RwLock<CatalogState>,
    pub(super) app: Option<tauri::AppHandle>,
}

impl Default for UiCatalog {
    fn default() -> Self {
        Self {
            state: RwLock::new(CatalogState::default()),
            app: None,
        }
    }
}

impl UiCatalog {
    pub(super) fn with_app(app: tauri::AppHandle) -> Self {
        Self {
            state: RwLock::new(CatalogState::default()),
            app: Some(app),
        }
    }

    pub(super) fn apply(&self, updates: Vec<UiCatalogUpdate>) -> Result<UiCatalogApply, String> {
        if updates.len() > super::types::MAX_EXTENSIONS {
            return Err(unavailable());
        }
        let mut state = self.state.write().map_err(|_| unavailable())?;
        validate_updates(&state, &updates)?;
        let has_updates = !updates.is_empty();
        let next_revision = if has_updates {
            Some(state.revision.checked_add(1).ok_or_else(unavailable)?)
        } else {
            None
        };
        let mut rejected_extensions = BTreeSet::new();
        for update in updates {
            state.extensions.remove(&update.extension_id);
            if update.entries.is_empty() {
                continue;
            }
            let extension_id = update.extension_id.clone();
            state.extensions.insert(
                extension_id.clone(),
                StoredCatalog {
                    identity: update.identity,
                    generation: update.generation,
                    catalog_revision: next_revision.ok_or_else(unavailable)?,
                    entries: update.entries,
                },
            );
            if super::ui_catalog_limits::validate(
                &state.extensions,
                next_revision.unwrap_or(state.revision),
            )
            .is_err()
            {
                state.extensions.remove(&extension_id);
                rejected_extensions.insert(extension_id);
            }
        }
        if let Some(revision) = next_revision {
            state.revision = revision;
            emit_changed(self.app.as_ref(), state.revision);
        }
        Ok(UiCatalogApply {
            revision: state.revision,
            rejected_extensions,
        })
    }

    pub(super) fn snapshot(&self) -> Result<UiCatalogSnapshot, String> {
        let state = self.state.read().map_err(|_| unavailable())?;
        Ok(super::ui_catalog_limits::snapshot(
            state.revision,
            &state.extensions,
        ))
    }

    #[cfg(test)]
    pub(super) fn replace(
        &self,
        identity: &HostIdentity,
        entries: Vec<UiCatalogEntry>,
    ) -> Result<u64, String> {
        let extension_id = entries
            .first()
            .map(|entry| entry.extension_id.clone())
            .ok_or_else(unavailable)?;
        self.apply(vec![UiCatalogUpdate {
            identity: identity.clone(),
            generation: 1,
            extension_id,
            entries,
        }])
        .map(|result| result.revision)
    }

    #[cfg(test)]
    pub(super) fn authorize(
        &self,
        extension_id: &str,
        contribution_id: &str,
        action_id: &str,
    ) -> Result<(), String> {
        self.route(extension_id, contribution_id, action_id)
            .map(|_| ())
    }
}

fn validate_updates(state: &CatalogState, updates: &[UiCatalogUpdate]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for update in updates {
        if update.generation == 0
            || !ids.insert(update.extension_id.as_str())
            || state
                .retired
                .get(&update.identity)
                .is_some_and(|retired| *retired >= update.generation)
            || update.entries.iter().any(|entry| {
                entry.extension_id != update.extension_id
                    || !identity_owns(&update.identity, &update.extension_id)
            })
        {
            return Err(unavailable());
        }
    }
    Ok(())
}

fn identity_owns(identity: &HostIdentity, extension_id: &str) -> bool {
    match identity {
        HostIdentity::Official => extension_id.starts_with("beaver."),
        HostIdentity::ThirdParty(id) => id == extension_id,
    }
}

pub(super) fn emit_changed(app: Option<&tauri::AppHandle>, revision: u64) {
    if let Some(app) = app {
        let _ = app.emit(CHANGED_EVENT, revision);
    }
}

pub(super) fn denied() -> String {
    "ui_action_denied".to_string()
}

pub(super) fn unavailable() -> String {
    super::error_codes::OPERATION_FAILED.to_string()
}
