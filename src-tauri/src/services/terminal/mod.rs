pub mod cwd_resolver;
mod manager;
mod owned_session;
pub mod pty_session;
mod public_error;
mod shutdown;
pub mod tab_store;

pub use manager::{PtyChannelEvent, PtyManager};

#[cfg(test)]
mod cwd_resolver_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tab_store_tests;

fn generate_token() -> zeroize::Zeroizing<String> {
    use rand::RngCore;
    let mut bytes = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let token = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    bytes.fill(0);
    zeroize::Zeroizing::new(token)
}

fn verify_token(expected: &str, provided: &str) -> Result<(), String> {
    use subtle::ConstantTimeEq;

    let matches: bool = expected.as_bytes().ct_eq(provided.as_bytes()).into();
    matches
        .then_some(())
        .ok_or_else(|| "terminal-access-denied".to_string())
}

#[cfg(test)]
mod token_tests {
    use super::verify_token;

    #[test]
    fn terminal_token_accepts_only_the_exact_value() {
        assert_eq!(verify_token("0123456789abcdef", "0123456789abcdef"), Ok(()));
        assert_eq!(
            verify_token("0123456789abcdef", "0123456789abcdee"),
            Err("terminal-access-denied".to_string())
        );
        assert_eq!(
            verify_token("0123456789abcdef", "short"),
            Err("terminal-access-denied".to_string())
        );
    }
}
