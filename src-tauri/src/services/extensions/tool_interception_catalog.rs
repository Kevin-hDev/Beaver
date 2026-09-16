use super::host_identity::HostIdentity;
use super::types::MAX_INTERCEPTORS;
use std::collections::BTreeMap;
use std::sync::RwLock;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InterceptorRegistration {
    pub(super) extension_id: String,
    pub(super) identity: HostIdentity,
    pub(super) generation: u64,
}

#[derive(Clone, Default)]
pub(crate) struct InterceptionSnapshot {
    pub(super) entries: Vec<InterceptorRegistration>,
}

impl InterceptionSnapshot {
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn extension_ids(&self) -> Vec<&str> {
        self.entries
            .iter()
            .map(|entry| entry.extension_id.as_str())
            .collect()
    }
}

#[derive(Default)]
pub(super) struct InterceptorCatalog {
    entries: RwLock<BTreeMap<String, InterceptorRegistration>>,
}

impl InterceptorCatalog {
    pub(super) fn replace(&self, entries: Vec<InterceptorRegistration>) {
        debug_assert!(entries.len() <= MAX_INTERCEPTORS);
        if let Ok(mut current) = self.entries.write() {
            *current = entries
                .into_iter()
                .take(MAX_INTERCEPTORS)
                .map(|entry| (entry.extension_id.clone(), entry))
                .collect();
        }
    }

    pub(super) fn retire(&self, identity: &HostIdentity, generation: u64) {
        if let Ok(mut entries) = self.entries.write() {
            entries
                .retain(|_, entry| entry.identity != *identity || entry.generation != generation);
        }
    }

    pub(super) fn remove(&self, extension_id: &str) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(extension_id);
        }
    }

    pub(super) fn snapshot(&self, ordered_ids: &[String]) -> InterceptionSnapshot {
        let Ok(entries) = self.entries.read() else {
            return InterceptionSnapshot::default();
        };
        InterceptionSnapshot {
            entries: ordered_ids
                .iter()
                .filter_map(|id| entries.get(id).cloned())
                .take(MAX_INTERCEPTORS)
                .collect(),
        }
    }
}
