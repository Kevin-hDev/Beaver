use super::{browser_view_key::BrowserViewKey, session_types::MAX_BROWSER_TABS};

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
    stamp: RuntimeStamp,
    released: bool,
}

#[derive(Default)]
pub(super) struct RuntimeRevisionCache {
    entries: Vec<RuntimeRevision>,
}

impl RuntimeRevisionCache {
    pub(super) fn accept_update(&mut self, key: BrowserViewKey, stamp: RuntimeStamp) -> bool {
        self.accept(key, stamp, false)
    }

    pub(super) fn accept_release(&mut self, key: BrowserViewKey, stamp: RuntimeStamp) -> bool {
        self.accept(key, stamp, true)
    }

    fn accept(&mut self, key: BrowserViewKey, stamp: RuntimeStamp, released: bool) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key == key) {
            if stamp.epoch < entry.stamp.epoch
                || (stamp.epoch == entry.stamp.epoch
                    && (entry.released || stamp.revision <= entry.stamp.revision))
            {
                return false;
            }
            entry.stamp = stamp;
            entry.released = released;
            return true;
        }
        if self.entries.len() == MAX_BROWSER_TABS {
            self.entries.remove(0);
        }
        self.entries.push(RuntimeRevision {
            key,
            stamp,
            released,
        });
        true
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}
