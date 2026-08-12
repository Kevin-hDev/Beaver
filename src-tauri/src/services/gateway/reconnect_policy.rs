use std::time::Duration;
use tokio_util::sync::CancellationToken;

const INITIAL_DELAY: Duration = Duration::from_secs(1);
const MAX_DELAY: Duration = Duration::from_secs(30);
const STABLE_CONNECTION: Duration = Duration::from_secs(60);

pub(super) struct ReconnectPolicy {
    next: Duration,
}

impl ReconnectPolicy {
    pub(super) fn new() -> Self {
        Self {
            next: INITIAL_DELAY,
        }
    }

    pub(super) fn next_delay(&mut self) -> Duration {
        let delay = self.next;
        self.next = (self.next * 2).min(MAX_DELAY);
        delay
    }

    pub(super) fn record_connection(&mut self, lifetime: Duration) {
        if lifetime >= STABLE_CONNECTION {
            self.next = INITIAL_DELAY;
        }
    }

    pub(super) async fn wait(&mut self, cancel: &CancellationToken) -> bool {
        let delay = self.next_delay();
        tokio::select! {
            _ = cancel.cancelled() => false,
            _ = tokio::time::sleep(delay) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReconnectPolicy;
    use std::time::Duration;

    #[test]
    fn reconnect_delay_grows_and_stops_at_the_shared_cap() {
        let mut policy = ReconnectPolicy::new();
        let delays: Vec<_> = (0..7).map(|_| policy.next_delay()).collect();

        assert_eq!(delays, [1, 2, 4, 8, 16, 30, 30].map(Duration::from_secs));
        policy.record_connection(Duration::from_secs(60));
        assert_eq!(policy.next_delay(), Duration::from_secs(1));
    }
}
