use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;

use super::types::{connection_index, CONNECTION_COUNT};

static EPOCHS: LazyLock<[AtomicU64; CONNECTION_COUNT]> =
    LazyLock::new(|| std::array::from_fn(|_| AtomicU64::new(0)));

pub fn current(connection_id: &str) -> Option<u64> {
    Some(EPOCHS[connection_index(connection_id)?].load(Ordering::SeqCst))
}

pub fn is_current(connection_id: &str, expected: u64) -> bool {
    current(connection_id).is_some_and(|current| current == expected)
}

pub fn invalidate(connection_id: &str) -> Option<u64> {
    let epoch = &EPOCHS[connection_index(connection_id)?];
    Some(epoch.fetch_add(1, Ordering::SeqCst).saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidation_rejects_the_previous_epoch() {
        let connection_id = "xai-oauth";
        let previous = current(connection_id).unwrap();
        invalidate(connection_id).unwrap();
        assert!(!is_current(connection_id, previous));
    }
}
