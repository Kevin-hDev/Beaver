use std::collections::HashMap;
use std::time::Instant;

const MAX_BUCKETS: usize = 10_000;
const BUCKET_TTL_SECS: u64 = 300;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RateLimitKey {
    pub channel_id: String,
    pub account_id: String,
    pub user_id: String,
}

struct Bucket {
    count: u32,
    window_start: Instant,
}

pub struct RateLimitDecision {
    pub allowed: bool,
    pub retry_after_ms: u64,
    pub remaining: u32,
}

pub struct RateLimiter {
    buckets: HashMap<RateLimitKey, Bucket>,
    max_per_window: u32,
    window_secs: u64,
    last_eviction: Instant,
}

impl RateLimiter {
    pub fn new(max_per_window: u32, window_secs: u64) -> Self {
        Self {
            buckets: HashMap::new(),
            max_per_window,
            window_secs,
            last_eviction: Instant::now(),
        }
    }

    pub fn consume(&mut self, key: &RateLimitKey) -> RateLimitDecision {
        self.maybe_evict();
        let now = Instant::now();

        let bucket = self.buckets.entry(key.clone()).or_insert(Bucket {
            count: 0,
            window_start: now,
        });

        let elapsed = now.duration_since(bucket.window_start).as_secs();
        if elapsed >= self.window_secs {
            bucket.count = 0;
            bucket.window_start = now;
        }

        if bucket.count >= self.max_per_window {
            let retry_after = self.window_secs.saturating_sub(elapsed) * 1000;
            return RateLimitDecision {
                allowed: false,
                retry_after_ms: retry_after,
                remaining: 0,
            };
        }

        bucket.count += 1;
        RateLimitDecision {
            allowed: true,
            retry_after_ms: 0,
            remaining: self.max_per_window - bucket.count,
        }
    }

    #[cfg(test)]
    pub fn reset(&mut self, key: &RateLimitKey) {
        self.buckets.remove(key);
    }

    fn maybe_evict(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_eviction).as_secs() < 60 {
            return;
        }
        self.last_eviction = now;
        self.buckets
            .retain(|_, b| now.duration_since(b.window_start).as_secs() < BUCKET_TTL_SECS);
        while self.buckets.len() > MAX_BUCKETS {
            if let Some(oldest_key) = self.find_oldest() {
                self.buckets.remove(&oldest_key);
            } else {
                break;
            }
        }
    }

    fn find_oldest(&self) -> Option<RateLimitKey> {
        self.buckets
            .iter()
            .min_by_key(|(_, b)| b.window_start)
            .map(|(k, _)| k.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key(user: &str) -> RateLimitKey {
        RateLimitKey {
            channel_id: "telegram".into(),
            account_id: "default".into(),
            user_id: user.into(),
        }
    }

    #[test]
    fn allows_within_limit() {
        let mut rl = RateLimiter::new(3, 60);
        let key = test_key("user1");
        assert!(rl.consume(&key).allowed);
        assert!(rl.consume(&key).allowed);
        assert!(rl.consume(&key).allowed);
    }

    #[test]
    fn blocks_over_limit() {
        let mut rl = RateLimiter::new(2, 60);
        let key = test_key("user1");
        assert!(rl.consume(&key).allowed);
        assert!(rl.consume(&key).allowed);
        let d = rl.consume(&key);
        assert!(!d.allowed);
        assert!(d.retry_after_ms > 0);
        assert_eq!(d.remaining, 0);
    }

    #[test]
    fn independent_per_user() {
        let mut rl = RateLimiter::new(1, 60);
        let k1 = test_key("alice");
        let k2 = test_key("bob");
        assert!(rl.consume(&k1).allowed);
        assert!(rl.consume(&k2).allowed);
        assert!(!rl.consume(&k1).allowed);
        assert!(!rl.consume(&k2).allowed);
    }

    #[test]
    fn remaining_decrements() {
        let mut rl = RateLimiter::new(5, 60);
        let key = test_key("user1");
        assert_eq!(rl.consume(&key).remaining, 4);
        assert_eq!(rl.consume(&key).remaining, 3);
        assert_eq!(rl.consume(&key).remaining, 2);
    }

    #[test]
    fn reset_clears_bucket() {
        let mut rl = RateLimiter::new(1, 60);
        let key = test_key("user1");
        assert!(rl.consume(&key).allowed);
        assert!(!rl.consume(&key).allowed);
        rl.reset(&key);
        assert!(rl.consume(&key).allowed);
    }
}
