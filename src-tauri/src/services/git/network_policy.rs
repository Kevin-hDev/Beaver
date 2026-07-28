use std::time::Duration;

const CONNECT_TIMEOUT_MS: i32 = 30_000;
const IDLE_TIMEOUT_MS: i32 = 30_000;
const TIMEOUT_CLASSIFICATION_MARGIN_MS: u64 = 1_000;

pub(crate) fn timeout_classification_threshold() -> Duration {
    let shortest = CONNECT_TIMEOUT_MS.min(IDLE_TIMEOUT_MS) as u64;
    Duration::from_millis(shortest.saturating_sub(TIMEOUT_CLASSIFICATION_MARGIN_MS))
}

/// # Safety
///
/// Doit être appelée une seule fois au point d'entrée, avant la création de threads.
pub unsafe fn configure_before_threads() -> Result<(), String> {
    unsafe {
        git2::opts::set_server_connect_timeout_in_milliseconds(CONNECT_TIMEOUT_MS)
            .map_err(|_| "politique réseau Git indisponible".to_string())?;
        git2::opts::set_server_timeout_in_milliseconds(IDLE_TIMEOUT_MS)
            .map_err(|_| "politique réseau Git indisponible".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_deadlines_are_positive_and_bounded() {
        assert!((1_000..=60_000).contains(&CONNECT_TIMEOUT_MS));
        assert!((1_000..=60_000).contains(&IDLE_TIMEOUT_MS));
    }

    #[test]
    fn timeout_classification_starts_just_before_the_network_deadline() {
        let threshold = timeout_classification_threshold();

        assert!(threshold < Duration::from_millis(CONNECT_TIMEOUT_MS as u64));
        assert!(threshold >= Duration::from_secs(1));
    }
}
