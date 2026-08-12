mod manager;
mod owned_session;
pub mod pty_session;
mod public_error;
mod shutdown;

pub use manager::{PtyChannelEvent, PtyManager};

#[cfg(test)]
mod tests;

fn generate_token() -> zeroize::Zeroizing<String> {
    use rand::RngCore;
    let mut bytes = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let token = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    bytes.fill(0);
    zeroize::Zeroizing::new(token)
}

fn verify_token(expected: &str, provided: &str) -> Result<(), String> {
    if expected.len() != provided.len() {
        return Err("terminal-access-denied".to_string());
    }
    let mismatch = expected
        .as_bytes()
        .iter()
        .zip(provided.as_bytes())
        .fold(0_u8, |accumulator, (left, right)| {
            accumulator | (left ^ right)
        });
    if mismatch == 0 {
        Ok(())
    } else {
        Err("terminal-access-denied".to_string())
    }
}
