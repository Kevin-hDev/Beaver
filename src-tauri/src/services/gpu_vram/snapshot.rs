use super::GpuVramSnapshot;
use std::sync::Mutex;

#[derive(Default)]
pub(super) struct SnapshotCache {
    value: Mutex<Option<GpuVramSnapshot>>,
}

impl SnapshotCache {
    pub(super) fn replace(&self, snapshot: Option<GpuVramSnapshot>) {
        if let Ok(mut value) = self.value.lock() {
            *value = snapshot;
        }
    }

    pub(super) fn get(&self) -> Option<GpuVramSnapshot> {
        self.value.lock().ok().and_then(|value| *value)
    }
}
