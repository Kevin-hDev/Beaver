use super::GpuMemorySnapshot;
use std::sync::Mutex;

#[derive(Default)]
pub(super) struct SnapshotCache {
    value: Mutex<Option<GpuMemorySnapshot>>,
}

impl SnapshotCache {
    pub(super) fn publish(&self, snapshot: Option<GpuMemorySnapshot>) {
        let Some(snapshot) = snapshot else {
            return;
        };
        self.replace(Some(snapshot));
    }

    pub(super) fn replace(&self, snapshot: Option<GpuMemorySnapshot>) {
        if let Ok(mut value) = self.value.lock() {
            *value = snapshot;
        }
    }

    pub(super) fn get(&self) -> Option<GpuMemorySnapshot> {
        self.value.lock().ok().and_then(|value| *value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::gpu_vram::GpuMemoryKind;

    #[test]
    fn a_failed_refresh_keeps_the_last_successful_measurement() {
        let cache = SnapshotCache::default();
        let measurement = GpuMemorySnapshot {
            kind: GpuMemoryKind::Dedicated,
            total_mb: 16_384,
            used_mb: Some(4_096),
        };

        cache.publish(Some(measurement));
        cache.publish(None);

        assert_eq!(cache.get(), Some(measurement));
    }
}
