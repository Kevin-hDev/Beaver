use super::{ServiceWorkKey, ServiceWorkPhase, ServiceWorkSupervisor, WorkRegistry};
use std::time::Instant;
use tokio::task::JoinHandle;

struct StopBatch<const CAPACITY: usize> {
    registry: WorkRegistry<CAPACITY>,
    handles: Vec<(ServiceWorkKey, JoinHandle<()>)>,
    armed: bool,
}

impl<const CAPACITY: usize> StopBatch<CAPACITY> {
    fn new(
        registry: WorkRegistry<CAPACITY>,
        handles: Vec<(ServiceWorkKey, JoinHandle<()>)>,
    ) -> Self {
        Self {
            registry,
            handles,
            armed: true,
        }
    }

    async fn wait_until(&mut self, deadline: Instant) -> bool {
        while let Some((_, handle)) = self.handles.last_mut() {
            if tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), handle)
                .await
                .is_err()
            {
                return false;
            }
            let (key, _) = self.handles.pop().expect("completed handle remains owned");
            let _ = self.registry.release(key);
        }
        self.registry.wait_closed_until(deadline).await
    }

    fn abort_and_release(&mut self) {
        for (key, handle) in self.handles.drain(..) {
            handle.abort();
            let _ = self.registry.release(key);
        }
        self.armed = false;
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl<const CAPACITY: usize> Drop for StopBatch<CAPACITY> {
    fn drop(&mut self) {
        if self.armed {
            self.abort_and_release();
        }
    }
}

impl<const CAPACITY: usize> WorkRegistry<CAPACITY> {
    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        if self.phase() == ServiceWorkPhase::Closed {
            return true;
        }
        let Ok(owner) = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            self.inner.stop_owner.lock(),
        )
        .await
        else {
            return self.phase() == ServiceWorkPhase::Closed;
        };
        if self.phase() == ServiceWorkPhase::Closed {
            return true;
        }
        let handles = self.begin_stop();
        let mut batch = StopBatch::new(self.clone(), handles);
        let stopped = batch.wait_until(deadline).await;
        if stopped {
            batch.disarm();
        } else {
            batch.abort_and_release();
        }
        drop(owner);
        self.phase() == ServiceWorkPhase::Closed
    }

    fn begin_stop(&self) -> Vec<(ServiceWorkKey, JoinHandle<()>)> {
        let mut state = self.lock_state();
        if state.phase == ServiceWorkPhase::Open {
            state.phase = ServiceWorkPhase::Closing;
        }
        let mut handles = Vec::with_capacity(CAPACITY);
        for (index, slot) in state.slots.iter_mut().enumerate() {
            if slot.occupied {
                if let Some(handle) = slot.handle.take() {
                    handles.push((
                        ServiceWorkKey {
                            index,
                            generation: slot.generation,
                        },
                        handle,
                    ));
                }
            }
        }
        if state.diagnostics.active == 0 {
            state.phase = ServiceWorkPhase::Closed;
        }
        drop(state);
        self.inner.cancel.cancel();
        self.inner.changed.notify_waiters();
        handles
    }

    async fn wait_closed_until(&self, deadline: Instant) -> bool {
        loop {
            let changed = self.inner.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.phase() == ServiceWorkPhase::Closed {
                return true;
            }
            if tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), changed)
                .await
                .is_err()
            {
                return self.phase() == ServiceWorkPhase::Closed;
            }
        }
    }
}

impl<const CAPACITY: usize> ServiceWorkSupervisor<CAPACITY> {
    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.registry.stop_and_wait(deadline).await
    }
}
