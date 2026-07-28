const CONNECT_TIMEOUT_MS: i32 = 30_000;
const IDLE_TIMEOUT_MS: i32 = 30_000;

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
}
