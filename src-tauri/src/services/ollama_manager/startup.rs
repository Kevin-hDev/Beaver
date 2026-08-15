use super::error::OllamaErrorCode;
use tokio::sync::watch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StartupBarrierState {
    Pending,
    Ready,
    Blocked { code: OllamaErrorCode },
}

#[derive(Clone)]
pub(crate) struct OllamaStartupBarrier {
    state: watch::Sender<StartupBarrierState>,
}

impl OllamaStartupBarrier {
    pub(crate) fn new() -> Self {
        let (state, _) = watch::channel(StartupBarrierState::Pending);
        Self { state }
    }

    pub(crate) fn state(&self) -> StartupBarrierState {
        self.state.borrow().clone()
    }

    pub(crate) fn publish(&self, next: StartupBarrierState) {
        self.state.send_if_modified(|current| {
            if *current == next || matches!(current, StartupBarrierState::Ready) {
                false
            } else {
                *current = next.clone();
                true
            }
        });
    }

    pub(crate) async fn wait_ready(&self) -> StartupBarrierState {
        let mut receiver = self.state.subscribe();
        loop {
            let current = receiver.borrow().clone();
            if !matches!(current, StartupBarrierState::Pending) {
                return current;
            }
            if receiver.changed().await.is_err() {
                return StartupBarrierState::Blocked {
                    code: OllamaErrorCode::OllamaInternal,
                };
            }
        }
    }

    pub(crate) async fn wait_until_ready(&self) -> StartupBarrierState {
        let mut receiver = self.state.subscribe();
        loop {
            if matches!(*receiver.borrow(), StartupBarrierState::Ready) {
                return StartupBarrierState::Ready;
            }
            if receiver.changed().await.is_err() {
                return StartupBarrierState::Blocked {
                    code: OllamaErrorCode::OllamaInternal,
                };
            }
        }
    }

}
