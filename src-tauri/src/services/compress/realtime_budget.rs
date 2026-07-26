use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::compress::token_estimate;

const CHECK_INTERVAL_TOKENS: u32 = 32;

#[derive(Debug, Clone)]
pub struct RealtimeBudget {
    base_tokens: usize,
    limit_tokens: usize,
    next_check_at: u32,
}

impl RealtimeBudget {
    pub fn from_messages(configured_context: u64, messages: &[ChatMessage]) -> Option<Self> {
        let config = crate::services::config::read_config().ok()?.advanced;
        Self::new(
            config.compression_enabled,
            configured_context,
            config.compression_threshold,
            token_estimate::estimate_tokens(messages),
        )
    }

    pub fn new(
        enabled: bool,
        configured_context: u64,
        threshold_pct: u8,
        base_tokens: usize,
    ) -> Option<Self> {
        if !enabled || configured_context == 0 || threshold_pct == 0 {
            return None;
        }
        let limit_tokens = (configured_context as f64 * threshold_pct as f64 / 100.0) as usize;
        Some(Self {
            base_tokens,
            limit_tokens,
            next_check_at: CHECK_INTERVAL_TOKENS,
        })
    }

    pub fn should_interrupt(&mut self, generated_tokens: u32) -> bool {
        if generated_tokens < self.next_check_at {
            return false;
        }
        self.next_check_at = generated_tokens.saturating_add(CHECK_INTERVAL_TOKENS);
        self.base_tokens.saturating_add(generated_tokens as usize) >= self.limit_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_when_threshold_or_context_is_zero() {
        assert!(RealtimeBudget::new(true, 100_000, 0, 0).is_none());
        assert!(RealtimeBudget::new(true, 0, 85, 0).is_none());
        assert!(RealtimeBudget::new(false, 100_000, 85, 0).is_none());
    }

    #[test]
    fn interrupts_when_base_plus_generated_reaches_limit() {
        let mut budget = RealtimeBudget::new(true, 100_000, 85, 84_980).unwrap();
        assert!(!budget.should_interrupt(16));
        assert!(budget.should_interrupt(32));
    }

    #[test]
    fn checks_on_bounded_intervals() {
        let mut budget = RealtimeBudget::new(true, 100_000, 85, 84_000).unwrap();
        assert!(!budget.should_interrupt(31));
        assert!(!budget.should_interrupt(32));
        assert!(!budget.should_interrupt(63));
        assert!(!budget.should_interrupt(64));
    }
}
