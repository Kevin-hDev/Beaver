use std::time::{Duration, Instant};

const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(60);
const STABLE_THRESHOLD: Duration = Duration::from_secs(300);
const MAX_RETRIES: u32 = 10;

pub struct ChannelSupervisor {
    channel_id: String,
    account_id: String,
    backoff: Duration,
    retries: u32,
    last_start: Option<Instant>,
}

pub enum RestartDecision {
    Retry(Duration),
    GiveUp(String),
}

impl ChannelSupervisor {
    pub fn new(channel_id: &str, account_id: &str) -> Self {
        Self {
            channel_id: channel_id.to_string(),
            account_id: account_id.to_string(),
            backoff: INITIAL_BACKOFF,
            retries: 0,
            last_start: None,
        }
    }

    pub fn mark_started(&mut self) {
        self.last_start = Some(Instant::now());
    }

    pub fn on_error(&mut self, is_auth_error: bool) -> RestartDecision {
        if is_auth_error {
            return RestartDecision::GiveUp(format!(
                "{}:{} — erreur d'authentification, arrêt définitif",
                self.channel_id, self.account_id
            ));
        }

        if self.retries >= MAX_RETRIES {
            return RestartDecision::GiveUp(format!(
                "{}:{} — {} tentatives échouées, arrêt",
                self.channel_id, self.account_id, MAX_RETRIES
            ));
        }

        if let Some(start) = self.last_start {
            if start.elapsed() > STABLE_THRESHOLD {
                self.backoff = INITIAL_BACKOFF;
                self.retries = 0;
            }
        }

        self.retries += 1;
        let delay = self.backoff;
        self.backoff = (self.backoff * 2).min(MAX_BACKOFF);

        RestartDecision::Retry(delay)
    }

    #[cfg(test)]
    pub fn retries(&self) -> u32 {
        self.retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_supervisor() -> ChannelSupervisor {
        ChannelSupervisor::new("telegram", "bot1")
    }

    #[test]
    fn first_retry_has_1s_backoff() {
        let mut sv = make_supervisor();
        match sv.on_error(false) {
            RestartDecision::Retry(d) => assert_eq!(d, Duration::from_secs(1)),
            RestartDecision::GiveUp(_) => panic!("should retry"),
        }
    }

    #[test]
    fn backoff_doubles() {
        let mut sv = make_supervisor();
        sv.on_error(false);
        match sv.on_error(false) {
            RestartDecision::Retry(d) => assert_eq!(d, Duration::from_secs(2)),
            _ => panic!("should retry"),
        }
        match sv.on_error(false) {
            RestartDecision::Retry(d) => assert_eq!(d, Duration::from_secs(4)),
            _ => panic!("should retry"),
        }
    }

    #[test]
    fn backoff_caps_at_max() {
        let mut sv = make_supervisor();
        for _ in 0..8 {
            sv.on_error(false);
        }
        match sv.on_error(false) {
            RestartDecision::Retry(d) => assert!(d <= MAX_BACKOFF),
            _ => panic!("should retry"),
        }
    }

    #[test]
    fn gives_up_after_max_retries() {
        let mut sv = make_supervisor();
        for _ in 0..MAX_RETRIES {
            sv.on_error(false);
        }
        assert!(matches!(sv.on_error(false), RestartDecision::GiveUp(_)));
    }

    #[test]
    fn auth_error_gives_up_immediately() {
        let mut sv = make_supervisor();
        assert!(matches!(sv.on_error(true), RestartDecision::GiveUp(_)));
        assert_eq!(sv.retries(), 0);
    }
}
