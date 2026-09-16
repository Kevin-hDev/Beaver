use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

use super::host_identity::HostIdentity;

#[derive(Default)]
struct State {
    total: usize,
    by_identity: BTreeMap<HostIdentity, usize>,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::default()));

pub(super) struct Lease(HostIdentity);

pub(super) fn acquire(identity: &HostIdentity) -> Result<Lease, &'static str> {
    let mut state = STATE.lock().map_err(|_| "core_request_failed")?;
    let identity_count = state.by_identity.get(identity).copied().unwrap_or_default();
    if state.total >= super::types::MAX_MODEL_GENERATIONS
        || identity_count >= super::types::MAX_MODEL_GENERATIONS_PER_HOST_IDENTITY
    {
        return Err("core_saturated");
    }
    state.total += 1;
    state
        .by_identity
        .insert(identity.clone(), identity_count + 1);
    Ok(Lease(identity.clone()))
}

impl Drop for Lease {
    fn drop(&mut self) {
        let Ok(mut state) = STATE.lock() else {
            return;
        };
        state.total = state.total.saturating_sub(1);
        let remaining = state
            .by_identity
            .get(&self.0)
            .copied()
            .unwrap_or_default()
            .saturating_sub(1);
        if remaining == 0 {
            state.by_identity.remove(&self.0);
        } else {
            state.by_identity.insert(self.0.clone(), remaining);
        }
    }
}

#[cfg(test)]
pub(super) fn reset() {
    if let Ok(mut state) = STATE.lock() {
        *state = State::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_generation_quota_is_global_and_per_identity() {
        reset();
        let first = HostIdentity::ThirdParty("one".into());
        let _a = acquire(&first).unwrap();
        let _b = acquire(&first).unwrap();
        assert_eq!(acquire(&first).err(), Some("core_saturated"));

        let mut leases = Vec::new();
        for name in ["two", "three", "four"] {
            let identity = HostIdentity::ThirdParty(name.into());
            leases.push(acquire(&identity).unwrap());
            leases.push(acquire(&identity).unwrap());
        }
        assert_eq!(acquire(&HostIdentity::Official).err(), Some("core_saturated"));
        drop(leases);
        reset();
    }
}
