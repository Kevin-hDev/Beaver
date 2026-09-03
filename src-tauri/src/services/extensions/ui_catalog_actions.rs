use super::ui_catalog::{denied, UiActionRoute, UiCatalog};
use std::collections::BTreeSet;

impl UiCatalog {
    pub(super) fn route(
        &self,
        extension_id: &str,
        contribution_id: &str,
        action_id: &str,
    ) -> Result<UiActionRoute, String> {
        let state = self.state.read().map_err(|_| denied())?;
        let stored = state.extensions.get(extension_id).ok_or_else(denied)?;
        stored
            .entries
            .iter()
            .any(|entry| {
                entry.contribution_id == contribution_id
                    && entry
                        .action_ids
                        .iter()
                        .any(|candidate| candidate == action_id)
            })
            .then(|| UiActionRoute {
                identity: stored.identity.clone(),
                generation: stored.generation,
                catalog_revision: stored.catalog_revision,
            })
            .ok_or_else(denied)
    }

    pub(super) fn refresh_actions(
        &self,
        extension_id: &str,
        contribution_id: &str,
        generation: u64,
        catalog_revision: u64,
        dynamic_actions: Vec<String>,
    ) -> Result<(), String> {
        if dynamic_actions.len() > super::ui_contract::MAX_ACTIONS_PER_EXTENSION {
            return Err(denied());
        }
        let mut state = self.state.write().map_err(|_| denied())?;
        let stored = state.extensions.get_mut(extension_id).ok_or_else(denied)?;
        if stored.generation != generation || stored.catalog_revision != catalog_revision {
            return Err(denied());
        }
        let entry = stored
            .entries
            .iter()
            .find(|entry| entry.contribution_id == contribution_id)
            .ok_or_else(denied)?;
        let mut actions = entry
            .declared_action_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        for action in dynamic_actions {
            super::ui_view_validation::owned_id(extension_id, &action)?;
            if stored.entries.iter().any(|other| {
                other.contribution_id != contribution_id
                    && other
                        .action_ids
                        .iter()
                        .any(|candidate| candidate == &action)
            }) {
                return Err(denied());
            }
            actions.insert(action);
        }
        let mut total_actions = 0usize;
        for other in &stored.entries {
            let count = if other.contribution_id == contribution_id {
                actions.len()
            } else {
                other.action_ids.len()
            };
            total_actions = total_actions.checked_add(count).ok_or_else(denied)?;
        }
        if total_actions > super::ui_contract::MAX_ACTIONS_PER_EXTENSION {
            return Err(denied());
        }
        stored
            .entries
            .iter_mut()
            .find(|entry| entry.contribution_id == contribution_id)
            .ok_or_else(denied)?
            .action_ids = actions.into_iter().collect();
        Ok(())
    }
}
