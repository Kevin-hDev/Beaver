use crate::models::RateLimitConfig;
use crate::services::gateway::channels::InboundMessage;

use super::rate_limit::{RateLimitDecision, RateLimitKey, RateLimiter};

pub struct GatewayRateLimiters {
    per_user: RateLimiter,
    per_channel: RateLimiter,
    global: RateLimiter,
}

impl GatewayRateLimiters {
    pub fn new(config: &RateLimitConfig) -> Self {
        Self {
            per_user: RateLimiter::new(config.per_user_per_minute, 60),
            per_channel: RateLimiter::new(config.per_channel_per_minute, 60),
            global: RateLimiter::new(config.global_per_minute, 60),
        }
    }

    pub fn consume(&mut self, msg: &InboundMessage) -> RateLimitDecision {
        let user_key = RateLimitKey {
            channel_id: msg.channel_key.channel_id.clone(),
            account_id: msg.channel_key.account_id.clone(),
            user_id: msg.user_id.clone(),
        };
        let channel_key = RateLimitKey {
            channel_id: msg.channel_key.channel_id.clone(),
            account_id: msg.channel_key.account_id.clone(),
            user_id: "*".to_string(),
        };
        let global_key = RateLimitKey {
            channel_id: "*".to_string(),
            account_id: "*".to_string(),
            user_id: "*".to_string(),
        };

        for decision in [
            self.global.consume(&global_key),
            self.per_channel.consume(&channel_key),
            self.per_user.consume(&user_key),
        ] {
            if !decision.allowed {
                return decision;
            }
        }
        RateLimitDecision {
            allowed: true,
            retry_after_ms: 0,
            remaining: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RateLimitConfig;
    use crate::services::gateway::types::ChannelKey;

    fn config(per_user: u32, per_channel: u32, global: u32) -> RateLimitConfig {
        RateLimitConfig {
            per_user_per_minute: per_user,
            per_channel_per_minute: per_channel,
            global_per_minute: global,
        }
    }

    fn msg(user_id: &str) -> InboundMessage {
        InboundMessage {
            channel_key: ChannelKey {
                channel_id: "telegram".into(),
                account_id: "main".into(),
            },
            user_id: user_id.into(),
            content: "hello".into(),
            message_id: format!("msg-{user_id}"),
            chat_id: "chat".into(),
            thread_id: None,
            is_group: false,
            mentions_bot: false,
        }
    }

    #[test]
    fn blocks_user_over_limit() {
        let mut limits = GatewayRateLimiters::new(&config(1, 10, 10));
        assert!(limits.consume(&msg("u1")).allowed);
        assert!(!limits.consume(&msg("u1")).allowed);
        assert!(limits.consume(&msg("u2")).allowed);
    }

    #[test]
    fn blocks_global_over_limit() {
        let mut limits = GatewayRateLimiters::new(&config(10, 10, 1));
        assert!(limits.consume(&msg("u1")).allowed);
        assert!(!limits.consume(&msg("u2")).allowed);
    }
}
