use super::host_identity::HostIdentity;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub(super) struct CoreCallQuota(Arc<Mutex<BTreeMap<HostIdentity, usize>>>);

pub(super) struct CoreCallLease {
    quota: CoreCallQuota,
    identity: HostIdentity,
}

impl CoreCallQuota {
    pub(super) fn acquire(&self, identity: &HostIdentity) -> Option<CoreCallLease> {
        let mut counts = self.0.lock().unwrap_or_else(|error| error.into_inner());
        let count = counts.get(identity).copied().unwrap_or_default();
        if count >= super::types::MAX_CORE_CALLS_PER_HOST_IDENTITY {
            return None;
        }
        counts.insert(identity.clone(), count + 1);
        Some(CoreCallLease {
            quota: self.clone(),
            identity: identity.clone(),
        })
    }
}

impl Drop for CoreCallLease {
    fn drop(&mut self) {
        let mut counts = self
            .quota
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let remaining = counts
            .get(&self.identity)
            .copied()
            .unwrap_or_default()
            .saturating_sub(1);
        if remaining == 0 {
            counts.remove(&self.identity);
        } else {
            counts.insert(self.identity.clone(), remaining);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_identity_cannot_exhaust_the_global_core_call_pool() {
        let quota = CoreCallQuota::default();
        let first = HostIdentity::ThirdParty("first".into());
        let neighbor = HostIdentity::ThirdParty("neighbor".into());
        let leases = (0..super::super::types::MAX_CORE_CALLS_PER_HOST_IDENTITY)
            .map(|_| quota.acquire(&first).expect("identity slot"))
            .collect::<Vec<_>>();

        assert!(quota.acquire(&first).is_none());
        assert!(quota.acquire(&neighbor).is_some());
        drop(leases);
        assert!(quota.acquire(&first).is_some());
    }
}
