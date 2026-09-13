use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;
use std::time::Instant;

pub(super) const WEBSOCKET_COOLDOWN_MS: u64 = 5 * 60 * 1_000;
static PROCESS_START: LazyLock<Instant> = LazyLock::new(Instant::now);
static DISABLED_UNTIL_MS: AtomicU64 = AtomicU64::new(0);

pub(super) fn should_attempt() -> bool {
    !cooldown_active(elapsed_ms(), DISABLED_UNTIL_MS.load(Ordering::Relaxed))
}

pub(super) fn mark_unavailable() {
    DISABLED_UNTIL_MS.store(cooldown_deadline(elapsed_ms()), Ordering::Relaxed);
}

pub(super) fn mark_available() {
    DISABLED_UNTIL_MS.store(0, Ordering::Relaxed);
}

pub(super) fn elapsed_ms() -> u64 {
    u64::try_from(PROCESS_START.elapsed().as_millis()).unwrap_or(u64::MAX)
}

pub(super) fn cooldown_deadline(now_ms: u64) -> u64 {
    now_ms.saturating_add(WEBSOCKET_COOLDOWN_MS)
}

pub(super) fn cooldown_active(now_ms: u64, disabled_until_ms: u64) -> bool {
    disabled_until_ms != 0 && now_ms < disabled_until_ms
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WebSocketFailure {
    Cancelled,
    ProviderRejected {
        code: crate::services::llm::provider_error::ProviderErrorCode,
    },
    Unavailable {
        partial: bool,
    },
}

impl WebSocketFailure {
    pub(super) fn has_partial_output(self) -> bool {
        matches!(self, Self::Unavailable { partial: true })
    }
}

pub(super) fn accumulator_failure(error: &str, partial: bool) -> WebSocketFailure {
    use crate::services::llm::provider_error::ProviderErrorCode;

    let code = match error {
        "service_tier_unavailable" => ProviderErrorCode::ServiceTierUnavailable,
        "provider_request_rejected" => ProviderErrorCode::ProviderRequestRejected,
        _ => return WebSocketFailure::Unavailable { partial },
    };
    WebSocketFailure::ProviderRejected { code }
}
