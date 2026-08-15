use super::{
    browser_view_key::BrowserViewKey,
    session_types::{BrowserRuntimeTabUpdate, MAX_BROWSER_TABS},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RuntimeStamp {
    pub(super) epoch: u64,
    pub(super) revision: u64,
}

impl RuntimeStamp {
    pub(super) fn new(epoch: u64, revision: u64) -> Option<Self> {
        (epoch > 0 && revision > 0).then_some(Self { epoch, revision })
    }
}

struct RuntimeRevision {
    key: BrowserViewKey,
    epoch: u64,
    max_revision: u64,
    fields: FieldRevisions,
    released: bool,
}

#[derive(Default)]
struct FieldRevisions {
    title: u64,
    url: u64,
    loading: u64,
    can_go_back: u64,
    can_go_forward: u64,
}

#[derive(Default)]
pub(super) struct RuntimeRevisionCache {
    entries: Vec<RuntimeRevision>,
}

impl RuntimeRevisionCache {
    pub(super) fn filter_update(
        &mut self,
        key: BrowserViewKey,
        stamp: RuntimeStamp,
        update: &mut BrowserRuntimeTabUpdate,
    ) -> bool {
        let entry = self.entry(key, stamp.epoch);
        if stamp.epoch < entry.epoch || (stamp.epoch == entry.epoch && entry.released) {
            update.clear();
            return false;
        }
        if stamp.epoch > entry.epoch {
            entry.reset(stamp.epoch);
        }
        entry.max_revision = entry.max_revision.max(stamp.revision);
        entry.fields.filter(stamp.revision, update)
    }

    pub(super) fn accept_release(&mut self, key: BrowserViewKey, stamp: RuntimeStamp) -> bool {
        let entry = self.entry(key, stamp.epoch);
        if stamp.epoch < entry.epoch
            || (stamp.epoch == entry.epoch
                && (entry.released || stamp.revision <= entry.max_revision))
        {
            return false;
        }
        if stamp.epoch > entry.epoch {
            entry.reset(stamp.epoch);
        }
        entry.max_revision = stamp.revision;
        entry.released = true;
        true
    }

    fn entry(&mut self, key: BrowserViewKey, epoch: u64) -> &mut RuntimeRevision {
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            return &mut self.entries[index];
        }
        if self.entries.len() == MAX_BROWSER_TABS {
            self.entries.remove(0);
        }
        self.entries.push(RuntimeRevision {
            key,
            epoch,
            max_revision: 0,
            fields: FieldRevisions::default(),
            released: false,
        });
        self.entries.last_mut().expect("runtime revision inserted")
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}

impl RuntimeRevision {
    fn reset(&mut self, epoch: u64) {
        self.epoch = epoch;
        self.max_revision = 0;
        self.fields = FieldRevisions::default();
        self.released = false;
    }
}

impl FieldRevisions {
    fn filter(&mut self, revision: u64, update: &mut BrowserRuntimeTabUpdate) -> bool {
        accept_field(&mut update.title, &mut self.title, revision)
            | accept_field(&mut update.url, &mut self.url, revision)
            | accept_field(&mut update.loading, &mut self.loading, revision)
            | accept_field(&mut update.can_go_back, &mut self.can_go_back, revision)
            | accept_field(
                &mut update.can_go_forward,
                &mut self.can_go_forward,
                revision,
            )
    }
}

fn accept_field<Value>(value: &mut Option<Value>, latest: &mut u64, revision: u64) -> bool {
    if value.is_none() {
        return false;
    }
    if revision <= *latest {
        *value = None;
        return false;
    }
    *latest = revision;
    true
}

impl BrowserRuntimeTabUpdate {
    fn clear(&mut self) {
        self.title = None;
        self.url = None;
        self.loading = None;
        self.can_go_back = None;
        self.can_go_forward = None;
    }
}
