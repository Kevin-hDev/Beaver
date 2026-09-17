use super::host_identity::HostIdentity;
use super::ui_catalog::{emit_changed, unavailable, UiCatalog};

impl UiCatalog {
    pub(super) fn retire(&self, identity: &HostIdentity, generation: u64) -> Result<u64, String> {
        if generation == 0 {
            return Err(unavailable());
        }
        let mut state = self.state.write().map_err(|_| unavailable())?;
        let visible_change = state
            .extensions
            .values()
            .any(|stored| stored.identity == *identity && stored.generation <= generation);
        let next_revision = visible_change
            .then(|| state.revision.checked_add(1).ok_or_else(unavailable))
            .transpose()?;
        record_retirement(&mut state, identity, generation);
        state
            .extensions
            .retain(|_, stored| stored.identity != *identity || stored.generation > generation);
        if let Some(revision) = next_revision {
            state.revision = revision;
            emit_changed(self.app.as_ref(), revision);
        }
        Ok(state.revision)
    }
}

fn record_retirement(
    state: &mut super::ui_catalog::CatalogState,
    identity: &HostIdentity,
    generation: u64,
) {
    let retired = state.retired.entry(identity.clone()).or_default();
    *retired = (*retired).max(generation);
    state
        .retired_order
        .push_back((identity.clone(), generation));
    while state.retired_order.len() > super::types::MAX_HOST_PROCESSES {
        let Some((expired_identity, expired_generation)) = state.retired_order.pop_front() else {
            break;
        };
        if state.retired.get(&expired_identity) == Some(&expired_generation) {
            state.retired.remove(&expired_identity);
        }
    }
}
