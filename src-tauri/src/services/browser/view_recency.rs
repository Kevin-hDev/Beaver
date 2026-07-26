use super::{browser_view_key::BrowserViewKey, session_types::MAX_BROWSER_TABS};

#[derive(Default)]
pub(super) struct ViewRecency {
    entries: Vec<BrowserViewKey>,
}

impl ViewRecency {
    pub(super) fn touch(
        &mut self,
        key: BrowserViewKey,
        protected: Option<&BrowserViewKey>,
    ) -> Option<BrowserViewKey> {
        self.entries.retain(|entry| entry != &key);
        let evicted = if self.entries.len() == MAX_BROWSER_TABS {
            self.entries
                .iter()
                .position(|entry| protected != Some(entry))
                .map(|index| self.entries.remove(index))
        } else {
            None
        };
        if self.entries.len() < MAX_BROWSER_TABS {
            self.entries.push(key);
        }
        evicted
    }

    pub(super) fn remove(&mut self, key: &BrowserViewKey) {
        self.entries.retain(|entry| entry != key);
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}
