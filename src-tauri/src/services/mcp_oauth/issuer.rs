pub fn verify_callback_issuer(
    expected: &str,
    received: Option<&str>,
    required: bool,
) -> Result<(), String> {
    match received {
        Some(value) if value == expected => Ok(()),
        None if !required => Ok(()),
        _ => Err("émetteur OAuth invalide".to_string()),
    }
}

pub fn verify_metadata_issuer(expected: &str, received: &str) -> Result<(), String> {
    if expected == received {
        Ok(())
    } else {
        Err("émetteur OAuth invalide".to_string())
    }
}

#[cfg(test)]
#[path = "issuer_tests.rs"]
mod tests;
