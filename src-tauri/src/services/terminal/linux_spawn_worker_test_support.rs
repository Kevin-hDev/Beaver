use super::{LinuxSpawnWorker, SpawnRequest};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

impl LinuxSpawnWorker {
    pub(in crate::services::terminal) async fn terminate_for_test(&self) {
        let sender = self.sender().expect("live worker sender");
        let (completed, result) = oneshot::channel();
        sender
            .try_send(SpawnRequest::Terminate(completed))
            .expect("terminate worker");
        result.await.expect("worker termination");
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let finished = self
                .state
                .lock()
                .expect("worker state")
                .join
                .as_ref()
                .is_some_and(JoinHandle::is_finished);
            if finished {
                return;
            }
            assert!(Instant::now() < deadline, "worker termination timed out");
            tokio::task::yield_now().await;
        }
    }
}
