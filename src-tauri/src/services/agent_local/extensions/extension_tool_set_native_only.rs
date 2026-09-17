use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::services::extensions::error_codes;

const MAX_NATIVE_ONLY_SESSIONS: usize = 256;
static SESSIONS: LazyLock<Mutex<HashMap<String, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(super) struct NativeOnlyLease(String);

impl NativeOnlyLease {
    pub(super) fn acquire(session_id: &str) -> Result<Self, String> {
        let mut sessions = SESSIONS
            .lock()
            .map_err(|_| error_codes::STATE_UNAVAILABLE)?;
        if !sessions.contains_key(session_id) && sessions.len() >= MAX_NATIVE_ONLY_SESSIONS {
            return Err(error_codes::STATE_UNAVAILABLE.to_string());
        }
        let count = sessions.entry(session_id.to_string()).or_default();
        *count = count.checked_add(1).ok_or(error_codes::STATE_UNAVAILABLE)?;
        Ok(Self(session_id.to_string()))
    }
}

impl Drop for NativeOnlyLease {
    fn drop(&mut self) {
        let Ok(mut sessions) = SESSIONS.lock() else {
            // A poisoned guard stays closed; never reopen extension execution on failure.
            log::error!("[extensions] native_only_guard_unavailable");
            return;
        };
        if let Some(count) = sessions.get_mut(&self.0) {
            *count -= 1;
            if *count == 0 {
                sessions.remove(&self.0);
            }
        }
    }
}

pub(crate) fn native_only_for_session(session_id: &str) -> bool {
    SESSIONS
        .lock()
        .map_or(true, |sessions| sessions.contains_key(session_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_turns_remain_closed_until_the_last_lease_ends() {
        let id = uuid::Uuid::new_v4().to_string();
        let first = NativeOnlyLease::acquire(&id).unwrap();
        let second = NativeOnlyLease::acquire(&id).unwrap();
        assert!(native_only_for_session(&id));
        drop(first);
        assert!(native_only_for_session(&id));
        drop(second);
        assert!(!native_only_for_session(&id));
    }
}
