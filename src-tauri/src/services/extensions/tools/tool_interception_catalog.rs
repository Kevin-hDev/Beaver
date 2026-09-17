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
    #[cfg(test)]
    deny_for_test: bool,
}

impl InterceptionSnapshot {
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn test_non_empty() -> Self {
        Self {
            entries: vec![InterceptorRegistration {
                extension_id: "test.interceptor".into(),
                identity: HostIdentity::ThirdParty("test.interceptor".into()),
                generation: 1,
            }],
            deny_for_test: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn test_denied() -> Self {
        let mut snapshot = Self::test_non_empty();
        snapshot.deny_for_test = true;
        snapshot
    }

    #[cfg(test)]
    pub(super) fn denies_for_test(&self) -> bool {
        self.deny_for_test
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
    pub(super) fn len(&self) -> usize {
        self.entries
            .read()
            .map(|entries| entries.len())
            .unwrap_or(0)
    }

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

    pub(super) fn is_current(&self, entry: &InterceptorRegistration) -> bool {
        self.entries
            .read()
            .ok()
            .and_then(|entries| entries.get(&entry.extension_id).cloned())
            .as_ref()
            == Some(entry)
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
            #[cfg(test)]
            deny_for_test: false,
        }
    }
}
